use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Error, InputSource, PipMode, PipPosition, PipSize, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub monitor: MonitorConfig,
    pub hotkeys: HotkeysConfig,
    pub pip: PipConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct MonitorConfig {
    pub serial: String,
    pub fallback: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct HotkeysConfig {
    pub swap: String,
    pub pip_toggle: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct PipConfig {
    pub mode: PipMode,
    pub input: InputSource,
    pub size: PipSize,
    pub position: PipPosition,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            serial: String::new(),
            fallback: "first".to_string(),
        }
    }
}

impl Default for HotkeysConfig {
    fn default() -> Self {
        Self {
            swap: "Super+Shift+P".to_string(),
            pip_toggle: "Super+Shift+O".to_string(),
        }
    }
}

impl Default for PipConfig {
    fn default() -> Self {
        Self {
            mode: PipMode::Pip,
            input: InputSource::Usbc,
            size: PipSize::Small,
            position: PipPosition::RightBottom,
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)?;
        toml::from_str(&text).map_err(|source| Error::TomlDecode {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn default_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("pip-switch")
            .join("config.toml")
    }

    pub fn write_example(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, Self::example_toml())?;
        Ok(())
    }

    pub fn example_toml() -> &'static str {
        r#"# pip-switch config
#
# Hotkeys use global-hotkey syntax. On Fedora/Linux, the Windows key is "Super".
# Examples: "Super+Shift+P", "Ctrl+Alt+P", "Alt+P"

[monitor]
# Leave empty to use the first detected MSI monitor.
serial = ""
# Possible values: "first"
fallback = "first"

[hotkeys]
swap = "Super+Shift+P"
pip_toggle = "Super+Shift+O"

[pip]
# Possible values: "pip", "pbp"
mode = "pip"
# Possible values: "hdmi1", "hdmi2", "dp", "usbc"
input = "usbc"
# Possible values: "small", "medium", "large"
size = "small"
# Possible values: "left_top", "right_top", "left_bottom", "right_bottom"
position = "right_bottom"
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_example_config() {
        let config: Config = toml::from_str(
            r#"
            [monitor]
            serial = "abc"
            fallback = "first"

            [hotkeys]
            swap = "Super+Shift+P"
            pip_toggle = "Super+Shift+O"

            [pip]
            mode = "pip"
            input = "usbc"
            size = "small"
            position = "right_bottom"
            "#,
        )
        .unwrap();

        assert_eq!(config.monitor.serial, "abc");
        assert_eq!(config.pip.input, InputSource::Usbc);
    }

    #[test]
    fn example_config_documents_values_and_matches_defaults() {
        let text = Config::example_toml();
        assert!(text.contains(r#"swap = "Super+Shift+P""#));
        assert!(text.contains(r#"pip_toggle = "Super+Shift+O""#));
        assert!(text.contains(r#"input = "usbc""#));
        assert!(text.contains(r#"size = "small""#));
        assert!(text.contains(r#"position = "right_bottom""#));
        assert!(text.contains(r#"Possible values: "hdmi1", "hdmi2", "dp", "usbc""#));

        let parsed: Config = toml::from_str(text).unwrap();
        assert_eq!(parsed, Config::default());
    }
}
