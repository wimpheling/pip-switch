use crate::{
    build_read_packet, build_write_packet, decode_setting_value, parse_read_response,
    parse_write_ack, Config, Error, HidDeviceInfo, HidTransport, PipMode, Result, Setting,
    PACKET_SIZE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorSelection {
    pub device: HidDeviceInfo,
    pub multiple_without_serial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub device: HidDeviceInfo,
    pub model_raw: Option<String>,
    pub firmware_raw: Option<String>,
}

pub struct MonitorClient<T> {
    transport: T,
    config: Config,
    read_timeout_ms: i32,
}

impl<T: HidTransport> MonitorClient<T> {
    pub fn new(transport: T, config: Config) -> Self {
        Self {
            transport,
            config,
            read_timeout_ms: 1_000,
        }
    }

    pub fn with_timeout_ms(mut self, timeout_ms: i32) -> Self {
        self.read_timeout_ms = timeout_ms;
        self
    }

    pub fn list(&self) -> Result<Vec<HidDeviceInfo>> {
        Ok(self
            .transport
            .devices()?
            .into_iter()
            .filter(HidDeviceInfo::is_supported_msi_monitor)
            .collect())
    }

    pub fn select(&self) -> Result<MonitorSelection> {
        let devices = self.list()?;
        if devices.is_empty() {
            return Err(Error::NoMonitor);
        }

        if !self.config.monitor.serial.trim().is_empty() {
            let serial = self.config.monitor.serial.trim();
            let device = devices
                .into_iter()
                .find(|device| device.serial.as_deref() == Some(serial))
                .ok_or_else(|| Error::SerialNotFound(serial.to_string()))?;
            return Ok(MonitorSelection {
                device,
                multiple_without_serial: false,
            });
        }

        if self.config.monitor.fallback != "first" {
            return Err(Error::Message(format!(
                "unsupported monitor fallback {:?}; only \"first\" is currently supported",
                self.config.monitor.fallback
            )));
        }

        Ok(MonitorSelection {
            multiple_without_serial: devices.len() > 1,
            device: devices.into_iter().next().expect("checked non-empty"),
        })
    }

    pub fn identify(&self) -> Result<Identity> {
        let device = self.select()?.device;
        let model_raw = self.try_raw_read_from(&device, "00140").ok();
        let firmware_raw = self.try_raw_read_from(&device, "00150").ok();
        Ok(Identity {
            device,
            model_raw,
            firmware_raw,
        })
    }

    pub fn read_setting(&self, setting: Setting) -> Result<String> {
        self.raw_read(setting.command())
    }

    pub fn write_setting(&self, setting: Setting, value: &str) -> Result<String> {
        self.raw_write(setting.command(), value)
    }

    pub fn raw_read(&self, command: &str) -> Result<String> {
        let selection = self.select()?;
        self.try_raw_read_from(&selection.device, command)
    }

    pub fn raw_write(&self, command: &str, value: &str) -> Result<String> {
        self.raw_write_impl(command, value, false)?
            .ok_or(Error::EmptyResponse)
    }

    fn raw_write_allow_missing_ack(&self, command: &str, value: &str) -> Result<()> {
        self.raw_write_impl(command, value, true)?;
        Ok(())
    }

    fn raw_write_impl(
        &self,
        command: &str,
        value: &str,
        allow_empty_ack: bool,
    ) -> Result<Option<String>> {
        let selection = self.select()?;
        let mut connection = self.transport.open_path(&selection.device.path)?;
        let packet = build_write_packet(command, value)?;
        connection.write(&packet)?;
        let mut response = [0_u8; PACKET_SIZE];
        let count = match connection.read_timeout(&mut response, self.read_timeout_ms) {
            Ok(count) => count,
            Err(error) if allow_empty_ack && is_post_write_missing_ack_error(&error) => {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        if count == 0 && allow_empty_ack {
            return Ok(None);
        }
        match parse_write_ack(&response[..count]) {
            Ok(ack) => Ok(Some(ack)),
            Err(Error::EmptyResponse) if allow_empty_ack => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn swap(&self) -> Result<()> {
        self.raw_write_allow_missing_ack(Setting::DisplaySwitch.command(), "001")?;
        Ok(())
    }

    pub fn pip_on(&self) -> Result<()> {
        let pip = &self.config.pip;
        self.raw_write_allow_missing_ack(Setting::PipInput.command(), pip.input.encoded())?;
        self.raw_write_allow_missing_ack(Setting::PipSize.command(), pip.size.encoded())?;
        self.raw_write_allow_missing_ack(Setting::PipPosition.command(), pip.position.encoded())?;
        self.raw_write_allow_missing_ack(Setting::PipMode.command(), pip.mode.encoded())?;
        Ok(())
    }

    pub fn pip_off(&self) -> Result<()> {
        self.raw_write_allow_missing_ack(Setting::PipMode.command(), PipMode::Off.encoded())?;
        Ok(())
    }

    pub fn pip_toggle(&self) -> Result<()> {
        let raw = self.read_setting(Setting::PipMode)?;
        if raw.trim() == PipMode::Off.encoded()
            || decode_setting_value(Setting::PipMode, &raw) == "off"
        {
            self.pip_on()
        } else {
            self.pip_off()
        }
    }

    pub fn probe_pip(&self) -> Result<Vec<(Setting, String, String)>> {
        [
            Setting::PipMode,
            Setting::PipInput,
            Setting::PbpInput,
            Setting::PipSize,
            Setting::PipPosition,
        ]
        .into_iter()
        .map(|setting| {
            let raw = self.read_setting(setting)?;
            let decoded = decode_setting_value(setting, &raw);
            Ok((setting, raw, decoded))
        })
        .collect()
    }

    fn try_raw_read_from(&self, device: &HidDeviceInfo, command: &str) -> Result<String> {
        let mut connection = self.transport.open_path(&device.path)?;
        let packet = build_read_packet(command)?;
        connection.write(&packet)?;
        let mut response = [0_u8; PACKET_SIZE];
        let count = connection.read_timeout(&mut response, self.read_timeout_ms)?;
        parse_read_response(command, &response[..count])
    }
}

fn is_post_write_missing_ack_error(error: &Error) -> bool {
    match error {
        Error::Hid(hidapi::HidError::HidApiError { message }) => {
            let message = message.to_ascii_lowercase();
            message.contains("poll error") || message.contains("device disconnected")
        }
        Error::Hid(hidapi::HidError::IoError { error }) => matches!(
            error.kind(),
            std::io::ErrorKind::NotConnected
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe
        ),
        _ => false,
    }
}
