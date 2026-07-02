use std::{path::PathBuf, time::Duration};

#[cfg(not(feature = "hotkeys"))]
use std::thread;

use clap::Parser;
use pip_switch_core::{Config, HidapiTransport, MonitorClient, Result};
use tracing::{error, info, warn};

#[derive(Debug, Parser)]
#[command(author, version, about = "Background hotkey daemon for pip-switch")]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let config_path = args.config.unwrap_or_else(Config::default_path);
    let config = Config::load_or_default(&config_path)?;

    info!(path = %config_path.display(), "loaded config");
    run(config)
}

#[cfg(feature = "hotkeys")]
fn run(config: Config) -> Result<()> {
    use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

    let swap_hotkey: HotKey = config.hotkeys.swap.parse().map_err(|error| {
        pip_switch_core::Error::Message(format!("invalid swap hotkey: {error}"))
    })?;
    let pip_toggle_hotkey: HotKey = config.hotkeys.pip_toggle.parse().map_err(|error| {
        pip_switch_core::Error::Message(format!("invalid pip_toggle hotkey: {error}"))
    })?;

    let manager = GlobalHotKeyManager::new()
        .map_err(|error| pip_switch_core::Error::Message(error.to_string()))?;
    manager
        .register(swap_hotkey)
        .map_err(|error| pip_switch_core::Error::Message(error.to_string()))?;
    manager
        .register(pip_toggle_hotkey)
        .map_err(|error| pip_switch_core::Error::Message(error.to_string()))?;

    info!(hotkey = %config.hotkeys.swap, "registered swap hotkey");
    info!(hotkey = %config.hotkeys.pip_toggle, "registered PIP toggle hotkey");

    let tray = setup_tray();
    if tray.is_none() {
        warn!(
            "tray menu is not enabled in this build; use CLI commands for tray-equivalent actions"
        );
    }

    loop {
        match GlobalHotKeyEvent::receiver().recv_timeout(Duration::from_millis(250)) {
            Ok(event) if event.state == HotKeyState::Pressed && event.id == swap_hotkey.id() => {
                if let Err(error) = execute(&config, Action::Swap) {
                    error!(%error, "swap failed");
                }
            }
            Ok(event)
                if event.state == HotKeyState::Pressed && event.id == pip_toggle_hotkey.id() =>
            {
                if let Err(error) = execute(&config, Action::PipToggle) {
                    error!(%error, "PIP toggle failed");
                }
            }
            Ok(_) => {}
            Err(error) if error.is_timeout() => {}
            Err(error) => {
                return Err(pip_switch_core::Error::Message(format!(
                    "hotkey event loop failed: {error}"
                )));
            }
        }

        if let Some(tray) = &tray {
            while let Some(action) = tray.next_action() {
                match action {
                    Action::Swap | Action::PipToggle | Action::Probe => {
                        if let Err(error) = execute(&config, action) {
                            error!(%error, "tray action failed");
                        }
                    }
                    Action::OpenConfig => {
                        info!(path = %Config::default_path().display(), "config file path");
                    }
                    Action::Quit => return Ok(()),
                }
            }
        }
    }
}

#[cfg(not(feature = "hotkeys"))]
fn run(_config: Config) -> Result<()> {
    warn!("hotkey feature disabled; daemon is running as a no-op health process");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

#[cfg_attr(not(feature = "tray"), allow(dead_code))]
enum Action {
    Swap,
    PipToggle,
    Probe,
    OpenConfig,
    Quit,
}

fn execute(config: &Config, action: Action) -> Result<()> {
    let transport = HidapiTransport::new()?;
    let client = MonitorClient::new(transport, config.clone());
    match action {
        Action::Swap => {
            client.swap()?;
            info!("swap command sent");
        }
        Action::PipToggle => {
            client.pip_toggle()?;
            info!("PIP toggled");
        }
        Action::Probe => {
            for (setting, raw, decoded) in client.probe_pip()? {
                info!(setting = %setting, command = setting.command(), raw, decoded, "probe result");
            }
        }
        Action::OpenConfig | Action::Quit => {}
    }
    Ok(())
}

#[cfg(feature = "tray")]
struct TrayState {
    _tray_icon: tray_icon::TrayIcon,
}

#[cfg(feature = "tray")]
impl TrayState {
    fn next_action(&self) -> Option<Action> {
        let event = tray_icon::menu::MenuEvent::receiver().try_recv().ok()?;
        match event.id().0.as_str() {
            "swap" => Some(Action::Swap),
            "pip_toggle" => Some(Action::PipToggle),
            "probe" => Some(Action::Probe),
            "open_config" => Some(Action::OpenConfig),
            "quit" => Some(Action::Quit),
            _ => None,
        }
    }
}

#[cfg(not(feature = "tray"))]
struct TrayState;

#[cfg(not(feature = "tray"))]
impl TrayState {
    fn next_action(&self) -> Option<Action> {
        None
    }
}

#[cfg(feature = "tray")]
fn setup_tray() -> Option<TrayState> {
    use tray_icon::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        Icon, TrayIconBuilder,
    };

    let swap = MenuItem::with_id("swap", "Swap PIP Displays", true, None);
    let toggle = MenuItem::with_id("pip_toggle", "Toggle PIP", true, None);
    let probe = MenuItem::with_id("probe", "Probe Monitor", true, None);
    let open_config = MenuItem::with_id("open_config", "Open Config", true, None);
    let quit = MenuItem::with_id("quit", "Quit", true, None);
    let separator = PredefinedMenuItem::separator();
    let menu = Menu::with_items(&[&swap, &toggle, &probe, &open_config, &separator, &quit])
        .map_err(|error| warn!(%error, "failed to create tray menu"))
        .ok()?;

    let icon = Icon::from_rgba(tray_icon_rgba(), 16, 16)
        .map_err(|error| warn!(%error, "failed to create tray icon image"))
        .ok()?;
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("pip-switch")
        .with_icon(icon)
        .build()
        .map_err(|error| warn!(%error, "failed to create tray icon"))
        .ok()?;

    info!("tray menu initialized");
    Some(TrayState {
        _tray_icon: tray_icon,
    })
}

#[cfg(not(feature = "tray"))]
fn setup_tray() -> Option<TrayState> {
    None
}

#[cfg(feature = "tray")]
fn tray_icon_rgba() -> Vec<u8> {
    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            let is_border = x == 0 || y == 0 || x == 15 || y == 15;
            let is_pip = x >= 9 && x <= 13 && y >= 3 && y <= 7;
            let color = if is_border {
                [32, 38, 46, 255]
            } else if is_pip {
                [20, 184, 166, 255]
            } else {
                [245, 247, 250, 255]
            };
            rgba.extend_from_slice(&color);
        }
    }
    rgba
}
