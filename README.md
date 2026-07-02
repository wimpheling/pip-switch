# pip-switch

`pip-switch` is a small Rust utility for controlling MSI Modern MD342CQPW PIP/PBP actions through MSI's USB HID monitor protocol.

The v1 target is:

- `pip-switch swap` sends the monitor PIP/PBP Display Switch command.
- `pip-switch pip-toggle` turns PIP off, or restores a configured PIP layout when PIP is off.
- `pip-switch-daemon` registers global hotkeys for those actions where the platform supports them.

The command table follows the MSI HID protocol used by `couriersud/msigd`. Unsupported monitor writes can be risky, so raw writes require an explicit guard flag.

## Build

```sh
cargo build --workspace
cargo test --workspace
```

## CLI

```sh
pip-switch list
pip-switch identify
pip-switch probe --pip
pip-switch read pip_mode
pip-switch swap
pip-switch pip-on
pip-switch pip-off
pip-switch pip-toggle
pip-switch raw-read 00600
pip-switch raw-write --i-understand-risk 00650 001
```

Write the default config:

```sh
pip-switch write-example-config
```

Default config path:

- Linux: `$XDG_CONFIG_HOME/pip-switch/config.toml` or `~/.config/pip-switch/config.toml`
- macOS: `~/Library/Application Support/pip-switch/config.toml`
- Windows: `%APPDATA%\pip-switch\config.toml`

Example:

```toml
# pip-switch config
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
```

## Linux Permissions

The monitor control interface appears as a USB HID device, usually:

```text
1462:3fa4 Micro Star International MSI Gaming Controller
```

If `pip-switch list` detects the monitor but `pip-switch probe --pip` fails with permission denied, install a udev rule and replug the monitor's USB upstream cable.

### Fedora 44

Install expected build/runtime dependencies:

```sh
sudo dnf install \
  gcc \
  pkgconf-pkg-config \
  hidapi-devel \
  systemd-devel \
  gtk3-devel \
  libayatana-appindicator-gtk3-devel
```

Install the Fedora-safe udev rule. Fedora usually does not have a `plugdev` group, so use `TAG+="uaccess"` without `GROUP="plugdev"`:

```sh
printf '%s\n' \
'SUBSYSTEM=="usb", ATTR{idVendor}=="1462", ATTR{idProduct}=="3fa4", MODE="0660", TAG+="uaccess"' \
'KERNEL=="hidraw*", ATTRS{idVendor}=="1462", ATTRS{idProduct}=="3fa4", MODE="0660", TAG+="uaccess"' \
| sudo tee /etc/udev/rules.d/60-pip-switch-msi.rules >/dev/null
```

Reload rules, then unplug/replug the monitor USB-B or USB-C upstream cable:

```sh
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Verify:

```sh
getfacl /dev/hidraw1
pip-switch probe --pip
```

The hidraw number can change. Use `pip-switch list` to see the current path.

### Ubuntu/Debian

Install expected build/runtime dependencies:

```sh
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libudev-dev \
  libhidapi-dev
```

On desktop systems, the same `uaccess` rule is usually enough:

```sh
sudo cp packaging/udev/60-pip-switch-msi.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

If you prefer group-based access on a system with a `plugdev` group, use this variant:

```sh
printf '%s\n' \
'SUBSYSTEM=="usb", ATTR{idVendor}=="1462", ATTR{idProduct}=="3fa4", MODE="0660", GROUP="plugdev", TAG+="uaccess"' \
'KERNEL=="hidraw*", ATTRS{idVendor}=="1462", ATTRS{idProduct}=="3fa4", MODE="0660", GROUP="plugdev", TAG+="uaccess"' \
| sudo tee /etc/udev/rules.d/60-pip-switch-msi.rules >/dev/null
```

## Linux Hotkeys

`global-hotkey` supports Linux through X11. On Wayland, bind shortcuts in the compositor to CLI commands such as:

```sh
pip-switch swap
pip-switch pip-toggle
```

## Hardware Validation

Recommended smoke sequence:

```sh
pip-switch identify
pip-switch probe --pip
pip-switch pip-on
pip-switch swap
pip-switch pip-toggle
```

If MD342CQPW command values differ from the MD342CQP assumptions, capture the official MSI app HID traffic and update the setting table in `crates/pip-switch-core/src/settings.rs`.
