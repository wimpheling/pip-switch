mod config;
mod error;
mod monitor;
mod protocol;
mod settings;
mod transport;

pub use config::{Config, MonitorConfig, PipConfig};
pub use error::{Error, Result};
pub use monitor::{Identity, MonitorClient, MonitorSelection};
pub use protocol::{
    build_read_packet, build_write_packet, decode_response_text, parse_read_response,
    parse_write_ack, PACKET_SIZE, REPORT_ID,
};
pub use settings::{
    decode_setting_value, encode_setting_value, InputSource, PipMode, PipPosition, PipSize, Setting,
};
pub use transport::{HidConnection, HidDeviceInfo, HidTransport, HidapiTransport};

#[cfg(test)]
pub mod mock;
