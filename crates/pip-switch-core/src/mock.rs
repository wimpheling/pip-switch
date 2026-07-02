use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use crate::{HidConnection, HidDeviceInfo, HidTransport, Result};

#[derive(Debug, Clone, Default)]
pub struct MockTransport {
    state: Arc<Mutex<MockState>>,
}

#[derive(Debug, Default)]
struct MockState {
    devices: Vec<HidDeviceInfo>,
    responses: HashMap<String, VecDeque<MockRead>>,
    writes: Vec<Vec<u8>>,
}

#[derive(Debug)]
enum MockRead {
    Response(Vec<u8>),
    Error(crate::Error),
}

impl MockTransport {
    pub fn new(devices: Vec<HidDeviceInfo>) -> Self {
        let state = MockState {
            devices,
            ..MockState::default()
        };
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn push_response(&self, path: impl Into<String>, response: impl Into<Vec<u8>>) {
        let mut state = self.state.lock().expect("mock mutex poisoned");
        state
            .responses
            .entry(path.into())
            .or_default()
            .push_back(MockRead::Response(response.into()));
    }

    pub fn push_read_error(&self, path: impl Into<String>, error: crate::Error) {
        let mut state = self.state.lock().expect("mock mutex poisoned");
        state
            .responses
            .entry(path.into())
            .or_default()
            .push_back(MockRead::Error(error));
    }

    pub fn writes(&self) -> Vec<Vec<u8>> {
        self.state
            .lock()
            .expect("mock mutex poisoned")
            .writes
            .clone()
    }
}

impl HidTransport for MockTransport {
    fn devices(&self) -> Result<Vec<HidDeviceInfo>> {
        Ok(self
            .state
            .lock()
            .expect("mock mutex poisoned")
            .devices
            .clone())
    }

    fn open_path(&self, path: &str) -> Result<Box<dyn HidConnection>> {
        Ok(Box::new(MockConnection {
            path: path.to_string(),
            state: Arc::clone(&self.state),
        }))
    }
}

struct MockConnection {
    path: String,
    state: Arc<Mutex<MockState>>,
}

impl HidConnection for MockConnection {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        self.state
            .lock()
            .expect("mock mutex poisoned")
            .writes
            .push(bytes.to_vec());
        Ok(bytes.len())
    }

    fn read_timeout(&mut self, bytes: &mut [u8], _timeout_ms: i32) -> Result<usize> {
        let mut state = self.state.lock().expect("mock mutex poisoned");
        let read = state
            .responses
            .get_mut(&self.path)
            .and_then(VecDeque::pop_front)
            .unwrap_or_else(|| MockRead::Response(b"\x015600\r".to_vec()));
        let response = match read {
            MockRead::Response(response) => response,
            MockRead::Error(error) => return Err(error),
        };
        let count = response.len().min(bytes.len());
        bytes[..count].copy_from_slice(&response[..count]);
        Ok(count)
    }
}

pub fn msi_device(path: &str, serial: Option<&str>) -> HidDeviceInfo {
    HidDeviceInfo {
        path: path.to_string(),
        vendor_id: 0x1462,
        product_id: 0x3fa4,
        manufacturer: Some("MSI".to_string()),
        product: Some("MSI Gaming Controller".to_string()),
        serial: serial.map(ToOwned::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use crate::{Config, MonitorClient, Setting};

    use super::*;

    #[test]
    fn lists_supported_devices() {
        let transport = MockTransport::new(vec![
            msi_device("a", None),
            HidDeviceInfo {
                path: "b".to_string(),
                vendor_id: 1,
                product_id: 2,
                manufacturer: None,
                product: None,
                serial: None,
            },
        ]);
        let client = MonitorClient::new(transport, Config::default());
        assert_eq!(client.list().unwrap().len(), 1);
    }

    #[test]
    fn reads_setting() {
        let transport = MockTransport::new(vec![msi_device("a", None)]);
        transport.push_response("a", b"\x015a00600001\r".to_vec());
        let client = MonitorClient::new(transport, Config::default());
        assert_eq!(client.read_setting(Setting::PipMode).unwrap(), "001");
    }

    #[test]
    fn writes_swap() {
        let transport = MockTransport::new(vec![msi_device("a", None)]);
        let client = MonitorClient::new(transport.clone(), Config::default());
        client.swap().unwrap();
        let writes = transport.writes();
        assert_eq!(&writes[0][..12], b"\x015b00650001\r");
    }

    #[test]
    fn swap_tolerates_post_write_disconnect() {
        let transport = MockTransport::new(vec![msi_device("a", None)]);
        transport.push_read_error(
            "a",
            crate::Error::Hid(hidapi::HidError::HidApiError {
                message: "unexpected poll error (device disconnected)".to_string(),
            }),
        );
        let client = MonitorClient::new(transport.clone(), Config::default());

        client.swap().unwrap();

        let writes = transport.writes();
        assert_eq!(&writes[0][..12], b"\x015b00650001\r");
    }

    #[test]
    fn raw_write_keeps_post_write_disconnect_strict() {
        let transport = MockTransport::new(vec![msi_device("a", None)]);
        transport.push_read_error(
            "a",
            crate::Error::Hid(hidapi::HidError::HidApiError {
                message: "unexpected poll error (device disconnected)".to_string(),
            }),
        );
        let client = MonitorClient::new(transport, Config::default());

        assert!(client.raw_write("00650", "001").is_err());
    }

    #[test]
    fn toggles_pip_off_when_currently_on() {
        let transport = MockTransport::new(vec![msi_device("a", None)]);
        transport.push_response("a", b"\x015a00600001\r".to_vec());
        transport.push_response("a", b"\x015600\r".to_vec());
        let client = MonitorClient::new(transport.clone(), Config::default());
        client.pip_toggle().unwrap();
        let writes = transport.writes();
        assert_eq!(&writes[1][..12], b"\x015b00600000\r");
    }

    #[test]
    fn pip_on_tolerates_empty_layout_write_acks() {
        let transport = MockTransport::new(vec![msi_device("a", None)]);
        transport.push_response("a", Vec::new());
        transport.push_response("a", Vec::new());
        transport.push_response("a", b"\x015600+\r".to_vec());
        transport.push_response("a", b"\x015600+\r".to_vec());

        let client = MonitorClient::new(transport.clone(), Config::default());
        client.pip_on().unwrap();

        let writes = transport.writes();
        assert_eq!(&writes[0][..12], b"\x015b00610004\r");
        assert_eq!(&writes[1][..12], b"\x015b00630000\r");
        assert_eq!(&writes[2][..12], b"\x015b00640003\r");
        assert_eq!(&writes[3][..12], b"\x015b00600001\r");
    }
}
