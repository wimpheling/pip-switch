use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML decode error in {path}: {source}")]
    TomlDecode {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("TOML encode error: {0}")]
    TomlEncode(#[from] toml::ser::Error),
    #[error("invalid command {0:?}; expected five ASCII hex characters")]
    InvalidCommand(String),
    #[error("invalid value {0:?}; expected ASCII value bytes")]
    InvalidValue(String),
    #[error("no matching MSI HID monitor found")]
    NoMonitor,
    #[error("configured monitor serial {0:?} was not found")]
    SerialNotFound(String),
    #[error("HID read returned no data")]
    EmptyResponse,
    #[error("monitor returned a write failure response: {0}")]
    WriteRejected(String),
    #[error("unknown setting {0:?}")]
    UnknownSetting(String),
    #[error("unsupported value {value:?} for setting {setting}")]
    UnsupportedValue {
        setting: &'static str,
        value: String,
    },
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;
