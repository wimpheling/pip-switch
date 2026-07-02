# MSI MD342CQPW PIP Switch App Plan

## Summary

Build a small cross-platform utility for an MSI Modern MD342CQPW monitor that can trigger PIP/PBP actions from keyboard shortcuts.

The app should support:

- A hotkey to swap the main and PIP/PBP displays.
- A hotkey to toggle PIP on/off using a deterministic configured layout.
- A CLI for testing, scripting, probing, and fallback shortcut binding.
- Native builds for Linux, Windows, and macOS.
- Explicit Fedora 44 support.

The implementation should use MSI's USB HID monitor protocol, reverse-engineered by `couriersud/msigd`, rather than DDC/CI. DDC/CI is unlikely to expose MSI's PIP/PBP display-switch behavior.

## Research Findings

Relevant references:

- `couriersud/msigd`: <https://github.com/couriersud/msigd>
- Open MD342CQP support PR: <https://github.com/couriersud/msigd/pull/71>
- MSI MD342CQPW product page: <https://www.msi.com/Business-Productivity-Monitor/Modern-MD342CQPW>
- MSI PIP/PBP support article: <https://www.msi.com/support/technical_details/MNT_PIP_PBP>
- Rust `hidapi`: <https://github.com/ruabmbua/hidapi-rs>
- Rust `global-hotkey`: <https://github.com/tauri-apps/global-hotkey>
- Rust `tray-icon`: <https://github.com/tauri-apps/tray-icon>

Key protocol facts from `msigd`:

- MSI monitor USB HID device:
  - Vendor ID: `0x1462`
  - Product ID: `0x3fa4`
  - Product string is commonly `MSI Gaming Controller`.
- HID packets are 64 bytes and start with report ID `0x01`.
- Read command shape:
  - `\x01 + "58" + command + "\r"`
- Write command shape:
  - `\x01 + "5b" + command + encoded_value + "\r"`
- Successful writes usually return `5600+`.

Open `msigd` PR #71 adds support for the closely related MD342CQP:

- `s140 = "00\xa0"`
- `s150 = "V20"`
- Input choices: `hdmi1`, `hdmi2`, `dp`, `usbc`
- KVM choices: `auto`, `upstream`, `type_c`

Treat the MD342CQPW as the same command family unless hardware probing shows otherwise.

## Product Scope

Version 1 should provide two actions:

1. `swap`
   - Trigger the monitor's PIP/PBP Display Switch function.
   - Expected command: `00650` with value `001`.
   - This is likely write-only.

2. `pip-toggle`
   - If PIP is off, enable PIP using the configured layout.
   - If PIP is on, turn it off.
   - Do not rely on the monitor remembering the last layout.

The PIP-on layout should be configured by the user:

- PIP mode: default `pip`
- PIP source: user-configured, for example `hdmi1`, `hdmi2`, `dp`, or `usbc`
- PIP size: user-configured
- PIP position: user-configured

## Architecture

Use a single Rust workspace.

Crates:

- `pip-switch-core`
  - HID discovery.
  - MSI command framing and response parsing.
  - Monitor identification.
  - Setting definitions and value encoding.
  - Config loading.
  - Error types.
  - Mockable HID transport for tests.

- `pip-switch`
  - CLI binary.
  - Used for hardware probing, manual control, scripting, and Wayland fallback shortcuts.

- `pip-switch-daemon`
  - Tray/background binary.
  - Owns global hotkeys.
  - Provides a minimal tray menu.

Recommended dependencies:

- `hidapi` for USB HID communication.
- `global-hotkey` for global shortcuts.
- `tray-icon` for system tray integration.
- `serde` and `toml` for config.
- `clap` for CLI argument parsing.
- `thiserror` for error handling.
- `tracing` and `tracing-subscriber` for logs.

## CLI Design

Commands:

```text
pip-switch list
pip-switch identify
pip-switch probe
pip-switch probe --pip
pip-switch read <setting>
pip-switch swap
pip-switch pip-on
pip-switch pip-off
pip-switch pip-toggle
pip-switch raw-read <command>
pip-switch raw-write --i-understand-risk <command> <value>
```

Behavior:

- `list` shows all matching MSI HID monitors.
- `identify` reads identity settings and prints model information.
- `probe --pip` reads likely PIP/PBP settings and prints raw and decoded values.
- `swap` sends Display Switch.
- `pip-on` writes configured PIP source, size, position, then enables PIP mode.
- `pip-off` writes PIP mode off.
- `pip-toggle` reads current PIP mode and calls `pip-on` or `pip-off`.
- Raw writes are guarded because unsupported MSI monitor writes may be risky.

## Daemon Design

The daemon should:

- Start in the background.
- Register two hotkeys.
- Expose a tray menu.
- Load config on startup.
- Log errors rather than crash when the monitor is disconnected.

Default hotkeys:

- Swap display: `Ctrl+Alt+P`
- Toggle PIP: `Ctrl+Alt+O`

Tray menu:

- Swap PIP Displays
- Toggle PIP
- Probe Monitor
- Open Config
- Quit

Linux caveat:

- `global-hotkey` supports Linux via X11 only.
- For Wayland, use the CLI with compositor-native shortcut bindings.

## Config

Create a TOML config file.

Example:

```toml
[monitor]
serial = ""
fallback = "first"

[hotkeys]
swap = "Ctrl+Alt+P"
pip_toggle = "Ctrl+Alt+O"

[pip]
mode = "pip"
input = "hdmi1"
size = "medium"
position = "right_top"
```

Rules:

- Prefer matching by serial when configured.
- If no serial is configured, use the first matching MSI monitor.
- If multiple monitors are found without a serial, warn in CLI output and daemon logs.

## MSI Settings To Validate

Likely settings from `msigd`:

| Command | Name | Expected values |
| --- | --- | --- |
| `00600` | PIP/PBP mode | `000=off`, `001=pip`, `002=pbp` |
| `00610` | PIP input | model-specific input enum |
| `00620` | PBP input | model-specific input enum |
| `00630` | PIP size | likely `small`, `medium`, `large` |
| `00640` | PIP position | likely corner positions |
| `00650` | Display Switch | likely write-only value `001` |
| `00660` | Audio Switch | likely write-only value `001`; not v1 |

Hardware validation sequence:

```sh
pip-switch identify
pip-switch probe --pip
pip-switch pip-on
pip-switch swap
pip-switch pip-toggle
```

If any command does not work on the MD342CQPW, capture the official MSI Windows app traffic:

1. Install USBPcap and Wireshark on Windows.
2. Start capture for the MSI HID device.
3. Trigger the same PIP/PBP action in the MSI app.
4. Save packet captures.
5. Extract the HID write payloads and update the command table.

## Cross-Platform Builds

Use GitHub Actions with native runners.

Native builds are preferred over cross-compiling because the project depends on HID, tray, global hotkeys, and eventually OS-specific packaging/signing.

CI matrix:

- `ubuntu-24.04`
- Fedora 44 container job
- `windows-latest`
- `macos-13` for `x86_64-apple-darwin`
- `macos-14` for `aarch64-apple-darwin`

Every CI job should run:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Use Rust build caching:

- Preferred: `Swatinem/rust-cache@v2`
- Configure it after Rust setup and before build/test steps.
- Keep `Cargo.lock` committed because this is an application.

## Fedora 44 Support

Fedora 44 should be explicitly documented and covered by CI in a Fedora container.

Expected Fedora dependencies:

```sh
sudo dnf install \
  gcc \
  pkgconf-pkg-config \
  hidapi-devel \
  systemd-devel \
  gtk3-devel \
  libayatana-appindicator-gtk3-devel
```

If package names differ on the final Fedora 44 image, adjust the docs and CI together.

Linux package artifacts should include a udev rule:

```udev
SUBSYSTEM=="usb", ATTR{idVendor}=="1462", ATTR{idProduct}=="3fa4", MODE="0660", GROUP="plugdev", TAG+="uaccess"
KERNEL=="hidraw*", ATTRS{idVendor}=="1462", ATTRS{idProduct}=="3fa4", MODE="0660", GROUP="plugdev", TAG+="uaccess"
```

For Fedora, group conventions may differ from Debian-based distributions. Validate whether `plugdev` exists by default. If not, prefer `TAG+="uaccess"` for logged-in desktop users and document any group-based fallback separately.

## GitHub Actions Workflows

Add `.github/workflows/ci.yml`:

- Trigger on push and pull request.
- Matrix for Ubuntu, Windows, macOS Intel, macOS Apple Silicon.
- Add a Fedora 44 container job.
- Install platform dependencies.
- Set up stable Rust.
- Use `Swatinem/rust-cache@v2`.
- Run format, clippy, tests, and release build.

Add `.github/workflows/release.yml`:

- Trigger on tags matching `v*`.
- Build release binaries per OS.
- Upload artifacts:
  - Linux generic `.tar.gz`
  - Fedora RPM or RPM-ready package metadata
  - Windows `.zip`
  - macOS `.zip` or `.tar.gz`
- Defer code signing and native installers until after v1 hardware behavior is proven.

## Testing

Unit tests:

- MSI packet encoding.
- MSI response parsing.
- Setting value encode/decode.
- Config parsing.
- Hotkey config parsing.

Mock HID tests:

- Monitor list.
- Monitor identify.
- Read setting.
- Write setting.
- Swap.
- PIP on.
- PIP off.
- PIP toggle.
- Error handling for disconnected monitor.

Manual hardware tests:

- Confirm monitor discovery.
- Confirm identity values.
- Confirm `probe --pip` output.
- Confirm `pip-on` creates the configured layout.
- Confirm `swap` swaps the displays.
- Confirm `pip-toggle` turns PIP off and then restores the configured layout.
- Confirm non-admin access works on Fedora 44 after installing udev rules.

## Risks

- Unsupported MSI monitor writes may be risky. Keep raw writes guarded and avoid broad probing writes.
- MD342CQPW may differ from MD342CQP despite likely shared command family.
- `00650` may be write-only and cannot be safely detected except by hardware behavior.
- Linux Wayland global hotkeys are not covered by `global-hotkey`; use CLI bindings there.
- Tray support on Linux depends on GTK/AppIndicator packages and desktop environment support.
- KVM switching may move USB control away from the host running the app; avoid changing KVM in v1.

## Implementation Milestones

1. Scaffold Rust workspace and CI.
2. Implement core HID transport and mock transport.
3. Implement monitor discovery and identity reads.
4. Implement setting table for known MD342-family settings.
5. Implement CLI `list`, `identify`, and `probe`.
6. Hardware-test PIP commands on the MD342CQPW.
7. Implement `swap`, `pip-on`, `pip-off`, and `pip-toggle`.
8. Implement daemon hotkeys and tray menu.
9. Add packaging artifacts and release workflow.
10. Perform manual hardware smoke test on Fedora 44, Windows, and macOS.

