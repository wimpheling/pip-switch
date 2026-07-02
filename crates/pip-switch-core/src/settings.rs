use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    PipMode,
    PipInput,
    PbpInput,
    PipSize,
    PipPosition,
    DisplaySwitch,
    AudioSwitch,
}

impl Setting {
    pub fn command(self) -> &'static str {
        match self {
            Self::PipMode => "00600",
            Self::PipInput => "00610",
            Self::PbpInput => "00620",
            Self::PipSize => "00630",
            Self::PipPosition => "00640",
            Self::DisplaySwitch => "00650",
            Self::AudioSwitch => "00660",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::PipMode => "pip_mode",
            Self::PipInput => "pip_input",
            Self::PbpInput => "pbp_input",
            Self::PipSize => "pip_size",
            Self::PipPosition => "pip_position",
            Self::DisplaySwitch => "display_switch",
            Self::AudioSwitch => "audio_switch",
        }
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Setting {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match normalized(value).as_str() {
            "pipmode" | "pip_mode" | "mode" | "00600" => Ok(Self::PipMode),
            "pipinput" | "pip_input" | "input" | "00610" => Ok(Self::PipInput),
            "pbpinput" | "pbp_input" | "00620" => Ok(Self::PbpInput),
            "pipsize" | "pip_size" | "size" | "00630" => Ok(Self::PipSize),
            "pipposition" | "pip_position" | "position" | "00640" => Ok(Self::PipPosition),
            "displayswitch" | "display_switch" | "swap" | "00650" => Ok(Self::DisplaySwitch),
            "audioswitch" | "audio_switch" | "00660" => Ok(Self::AudioSwitch),
            _ => Err(Error::UnknownSetting(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipMode {
    Off,
    Pip,
    Pbp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    Hdmi1,
    Hdmi2,
    Dp,
    Usbc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipPosition {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
}

pub fn encode_setting_value(setting: Setting, value: &str) -> Result<String> {
    if value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(value.to_ascii_uppercase());
    }

    match setting {
        Setting::PipMode => encode_pip_mode(value),
        Setting::PipInput | Setting::PbpInput => encode_input(value),
        Setting::PipSize => encode_size(value),
        Setting::PipPosition => encode_position(value),
        Setting::DisplaySwitch | Setting::AudioSwitch => match normalized(value).as_str() {
            "1" | "on" | "true" | "trigger" | "swap" => Ok("001".to_string()),
            _ => unsupported(setting, value),
        },
    }
}

pub fn decode_setting_value(setting: Setting, raw: &str) -> String {
    let raw = raw.trim();
    match setting {
        Setting::PipMode => match raw {
            "000" => "off".to_string(),
            "001" => "pip".to_string(),
            "002" => "pbp".to_string(),
            _ => raw.to_string(),
        },
        Setting::PipInput | Setting::PbpInput => match raw {
            "001" => "hdmi1".to_string(),
            "002" => "hdmi2".to_string(),
            "003" => "dp".to_string(),
            "004" => "usbc".to_string(),
            _ => raw.to_string(),
        },
        Setting::PipSize => match raw {
            "000" => "small".to_string(),
            "001" => "medium".to_string(),
            "002" => "large".to_string(),
            _ => raw.to_string(),
        },
        Setting::PipPosition => match raw {
            "000" => "left_top".to_string(),
            "001" => "right_top".to_string(),
            "002" => "left_bottom".to_string(),
            "003" => "right_bottom".to_string(),
            _ => raw.to_string(),
        },
        Setting::DisplaySwitch | Setting::AudioSwitch => raw.to_string(),
    }
}

impl PipMode {
    pub fn encoded(self) -> &'static str {
        match self {
            Self::Off => "000",
            Self::Pip => "001",
            Self::Pbp => "002",
        }
    }
}

impl InputSource {
    pub fn encoded(self) -> &'static str {
        match self {
            Self::Hdmi1 => "001",
            Self::Hdmi2 => "002",
            Self::Dp => "003",
            Self::Usbc => "004",
        }
    }
}

impl PipSize {
    pub fn encoded(self) -> &'static str {
        match self {
            Self::Small => "000",
            Self::Medium => "001",
            Self::Large => "002",
        }
    }
}

impl PipPosition {
    pub fn encoded(self) -> &'static str {
        match self {
            Self::LeftTop => "000",
            Self::RightTop => "001",
            Self::LeftBottom => "002",
            Self::RightBottom => "003",
        }
    }
}

fn encode_pip_mode(value: &str) -> Result<String> {
    match normalized(value).as_str() {
        "off" | "none" | "disable" | "disabled" => Ok("000".to_string()),
        "pip" => Ok("001".to_string()),
        "pbp" => Ok("002".to_string()),
        _ => unsupported(Setting::PipMode, value),
    }
}

fn encode_input(value: &str) -> Result<String> {
    match normalized(value).as_str() {
        "hdmi1" | "hdmi_1" => Ok("001".to_string()),
        "hdmi2" | "hdmi_2" => Ok("002".to_string()),
        "dp" | "displayport" | "display_port" => Ok("003".to_string()),
        "usbc" | "usb_c" | "typec" | "type_c" => Ok("004".to_string()),
        _ => unsupported(Setting::PipInput, value),
    }
}

fn encode_size(value: &str) -> Result<String> {
    match normalized(value).as_str() {
        "small" => Ok("000".to_string()),
        "medium" | "mid" => Ok("001".to_string()),
        "large" => Ok("002".to_string()),
        _ => unsupported(Setting::PipSize, value),
    }
}

fn encode_position(value: &str) -> Result<String> {
    match normalized(value).as_str() {
        "lefttop" | "left_top" | "top_left" => Ok("000".to_string()),
        "righttop" | "right_top" | "top_right" => Ok("001".to_string()),
        "leftbottom" | "left_bottom" | "bottom_left" => Ok("002".to_string()),
        "rightbottom" | "right_bottom" | "bottom_right" => Ok("003".to_string()),
        _ => unsupported(Setting::PipPosition, value),
    }
}

fn unsupported(setting: Setting, value: &str) -> Result<String> {
    Err(Error::UnsupportedValue {
        setting: setting.name(),
        value: value.to_string(),
    })
}

fn normalized(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_setting_names() {
        assert_eq!("pip-mode".parse::<Setting>().unwrap(), Setting::PipMode);
        assert_eq!("00650".parse::<Setting>().unwrap(), Setting::DisplaySwitch);
    }

    #[test]
    fn encodes_and_decodes_values() {
        assert_eq!(
            encode_setting_value(Setting::PipMode, "pip").unwrap(),
            "001"
        );
        assert_eq!(
            encode_setting_value(Setting::PipInput, "usbc").unwrap(),
            "004"
        );
        assert_eq!(
            decode_setting_value(Setting::PipPosition, "001"),
            "right_top"
        );
    }
}
