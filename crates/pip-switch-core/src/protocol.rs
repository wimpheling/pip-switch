use crate::{Error, Result};

pub const REPORT_ID: u8 = 0x01;
pub const PACKET_SIZE: usize = 64;

pub fn build_read_packet(command: &str) -> Result<[u8; PACKET_SIZE]> {
    validate_command(command)?;
    let mut packet = [0_u8; PACKET_SIZE];
    packet[0] = REPORT_ID;
    packet[1..3].copy_from_slice(b"58");
    packet[3..8].copy_from_slice(command.as_bytes());
    packet[8] = b'\r';
    Ok(packet)
}

pub fn build_write_packet(command: &str, value: &str) -> Result<[u8; PACKET_SIZE]> {
    validate_command(command)?;
    validate_value(value)?;
    let mut packet = [0_u8; PACKET_SIZE];
    let payload = format!("5b{command}{value}\r");
    if payload.len() + 1 > PACKET_SIZE {
        return Err(Error::InvalidValue(value.to_string()));
    }
    packet[0] = REPORT_ID;
    packet[1..1 + payload.len()].copy_from_slice(payload.as_bytes());
    Ok(packet)
}

pub fn decode_response_text(bytes: &[u8]) -> Result<String> {
    let payload = if bytes.first() == Some(&REPORT_ID) {
        &bytes[1..]
    } else {
        bytes
    };
    let end = payload
        .iter()
        .position(|byte| *byte == 0 || *byte == b'\r')
        .unwrap_or(payload.len());
    let text = String::from_utf8_lossy(&payload[..end]).trim().to_string();
    if text.is_empty() {
        return Err(Error::EmptyResponse);
    }
    Ok(text)
}

pub fn parse_write_ack(bytes: &[u8]) -> Result<String> {
    let text = decode_response_text(bytes)?;
    if text.starts_with("5600") {
        Ok(text)
    } else {
        Err(Error::WriteRejected(text))
    }
}

pub fn parse_read_response(command: &str, bytes: &[u8]) -> Result<String> {
    validate_command(command)?;
    let text = decode_response_text(bytes)?;
    if let Some(value) = text.strip_prefix(&format!("5a{command}")) {
        return Ok(value.to_string());
    }
    if let Some(value) = text.strip_prefix(&format!("5A{command}")) {
        return Ok(value.to_string());
    }
    if let Some(value) = text.strip_prefix(&format!("5b{command}")) {
        return Ok(value.to_string());
    }
    if let Some(value) = text.strip_prefix(&format!("5B{command}")) {
        return Ok(value.to_string());
    }
    if let Some(value) = text.strip_prefix(command) {
        return Ok(value.to_string());
    }
    if text.len() > 2 && text[..2].eq_ignore_ascii_case("5a") {
        return Ok(text[2..].to_string());
    }
    Ok(text)
}

fn validate_command(command: &str) -> Result<()> {
    if command.len() == 5 && command.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(Error::InvalidCommand(command.to_string()))
    }
}

fn validate_value(value: &str) -> Result<()> {
    if value.bytes().all(|byte| byte.is_ascii() && byte != 0) {
        Ok(())
    } else {
        Err(Error::InvalidValue(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_read_packet() {
        let packet = build_read_packet("00600").unwrap();
        assert_eq!(packet[0], 0x01);
        assert_eq!(&packet[1..9], b"5800600\r");
        assert!(packet[9..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn builds_write_packet() {
        let packet = build_write_packet("00650", "001").unwrap();
        assert_eq!(&packet[0..12], b"\x015b00650001\r");
        assert!(packet[12..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn parses_write_ack() {
        assert_eq!(parse_write_ack(b"\x015600\r\0").unwrap(), "5600");
        assert!(parse_write_ack(b"\x015601\r").is_err());
    }

    #[test]
    fn parses_read_response_shapes() {
        assert_eq!(
            parse_read_response("00600", b"\x015a00600001\r").unwrap(),
            "001"
        );
        assert_eq!(parse_read_response("00600", b"\x015a001\r").unwrap(), "001");
        assert_eq!(
            parse_read_response("00600", b"\x0100600001\r").unwrap(),
            "001"
        );
        assert_eq!(
            parse_read_response("00600", b"\x015b00600001\r").unwrap(),
            "001"
        );
    }
}
