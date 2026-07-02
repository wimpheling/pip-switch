# pip-switch

`pip-switch` controls MSI Modern MD342CQPW PIP/PBP actions through the monitor's USB HID protocol.

It provides:

- `pip-switch`, a CLI for direct use, scripts, and desktop shortcut bindings.
- `pip-switch-daemon`, a hotkey daemon for platforms where global hotkeys are supported.

The monitor must be connected to the computer with a USB upstream connection, not only HDMI/DisplayPort/USB-C video. The USB HID device usually appears as:

```text
1462:3fa4 Micro Star International MSI Gaming Controller
```

## Getting Started

### Installing

Download the latest artifact for your OS from GitHub Releases.

#### macOS

Install the `.pkg` artifact.

The package installs:

- `/Applications/pip-switch.app`
- `/usr/local/bin/pip-switch`
- `/Library/LaunchAgents/dev.pip-switch.daemon.plist`

Current macOS packages are unsigned. If macOS blocks installation or first launch, approve it in System Settings under Privacy & Security.

Test the CLI:

```sh
pip-switch list
pip-switch swap
pip-switch pip-toggle
```

Start or restart the daemon:

```sh
sudo launchctl unload /Library/LaunchAgents/dev.pip-switch.daemon.plist 2>/dev/null || true
sudo launchctl load /Library/LaunchAgents/dev.pip-switch.daemon.plist
```

If hotkeys do not fire, macOS may need Accessibility or Input Monitoring permission for `pip-switch-daemon`.

#### Windows

Install the `.msi` artifact.

Then open PowerShell or Windows Terminal and test:

```powershell
pip-switch list
pip-switch swap
pip-switch pip-toggle
```

The MSI also installs `pip-switch-daemon.exe`. Autostart and signed installer polish are planned follow-ups.

#### Fedora

Install the `.rpm` artifact:

```sh
sudo dnf install ./pip-switch-*.rpm
```

The RPM installs:

- `/usr/bin/pip-switch`
- `/usr/bin/pip-switch-daemon`
- `/usr/lib/udev/rules.d/60-pip-switch-msi.rules`
- `/usr/lib/systemd/user/pip-switch-daemon.service`

After installing, unplug/replug the monitor USB-B or USB-C upstream cable, or reboot, so udev permissions apply.

Test:

```sh
pip-switch list
pip-switch probe --pip
pip-switch swap
pip-switch pip-toggle
```

On Fedora Wayland, bind desktop shortcuts directly to:

```sh
pip-switch swap
pip-switch pip-toggle
```

On X11, the daemon can register configured hotkeys:

```sh
systemctl --user enable --now pip-switch-daemon.service
```

Disable it with:

```sh
systemctl --user disable --now pip-switch-daemon.service
```

#### Ubuntu/Debian

Install the `.deb` artifact:

```sh
sudo apt install ./pip-switch-*.deb
```

After installing, unplug/replug the monitor USB upstream cable, or reboot.

#### Generic Linux

Use the `.tar.gz` artifact if your distribution does not use RPM or DEB packages. Install the binaries and udev rule manually from the archive, then reload udev:

```sh
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Unplug/replug the monitor USB upstream cable after installing the rule.

### Configuring

Create the default config:

```sh
pip-switch write-example-config
```

Config paths:

- macOS: `~/Library/Application Support/pip-switch/config.toml`
- Windows: `%APPDATA%\pip-switch\config.toml`
- Linux: `$XDG_CONFIG_HOME/pip-switch/config.toml` or `~/.config/pip-switch/config.toml`

Open the config on macOS:

```sh
open "$HOME/Library/Application Support/pip-switch/config.toml"
```

Example config:

```toml
# pip-switch config
#
# Hotkeys use global-hotkey syntax.
# On Fedora/Linux, the Windows key is "Super".
# On macOS, use "Cmd", "Command", or "Super" for the Command key.

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

macOS hotkey examples:

```toml
[hotkeys]
swap = "Cmd+Shift+P"
pip_toggle = "Cmd+Shift+O"
```

After changing hotkeys, restart the daemon for the new config to load.

macOS:

```sh
sudo launchctl unload /Library/LaunchAgents/dev.pip-switch.daemon.plist 2>/dev/null || true
sudo launchctl load /Library/LaunchAgents/dev.pip-switch.daemon.plist
```

Linux systemd user service:

```sh
systemctl --user restart pip-switch-daemon.service
```

### Verifying

Recommended smoke test:

```sh
pip-switch list
pip-switch identify
pip-switch probe --pip
pip-switch pip-on
pip-switch swap
pip-switch pip-toggle
```

If `pip-switch list` detects the monitor but `probe --pip` fails with permission denied on Linux, check the hidraw path shown by `pip-switch list`, then inspect permissions:

```sh
getfacl /dev/hidraw1
```

The hidraw number can change.

## CLI Reference

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

Unsupported MSI monitor writes may be risky, so raw writes require `--i-understand-risk`.

## Linux Permissions

The packaged Fedora/RPM and Debian/Ubuntu installers include a udev rule. If you need to install it manually on Fedora, use the `uaccess` rule without `plugdev`:

```sh
printf '%s\n' \
'SUBSYSTEM=="usb", ATTR{idVendor}=="1462", ATTR{idProduct}=="3fa4", MODE="0660", TAG+="uaccess"' \
'KERNEL=="hidraw*", ATTRS{idVendor}=="1462", ATTRS{idProduct}=="3fa4", MODE="0660", TAG+="uaccess"' \
| sudo tee /etc/udev/rules.d/60-pip-switch-msi.rules >/dev/null

sudo udevadm control --reload-rules
sudo udevadm trigger
```

For Ubuntu/Debian systems with a `plugdev` group, this group-based variant is also valid:

```sh
printf '%s\n' \
'SUBSYSTEM=="usb", ATTR{idVendor}=="1462", ATTR{idProduct}=="3fa4", MODE="0660", GROUP="plugdev", TAG+="uaccess"' \
'KERNEL=="hidraw*", ATTRS{idVendor}=="1462", ATTRS{idProduct}=="3fa4", MODE="0660", GROUP="plugdev", TAG+="uaccess"' \
| sudo tee /etc/udev/rules.d/60-pip-switch-msi.rules >/dev/null
```

## Building

Fedora dependencies:

```sh
sudo dnf install \
  gcc \
  pkgconf-pkg-config \
  hidapi-devel \
  systemd-devel \
  gtk3-devel \
  libayatana-appindicator-gtk3-devel
```

Ubuntu/Debian dependencies:

```sh
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libudev-dev \
  libhidapi-dev
```

Build and test:

```sh
cargo build --workspace
cargo test --workspace
```

## Release Process

Release artifacts are built from tags named `vX.Y.Z`. The tag must match the Cargo workspace version.

Update the workspace version in `Cargo.toml`, commit it, then create a matching tag:

```sh
git tag v0.1.2
git push origin v0.1.2
```

The release workflow verifies the tag against the Cargo version, builds native artifacts on GitHub Actions, and publishes them to GitHub Releases.

Current artifacts:

- Fedora/RHEL: `.rpm`
- Ubuntu/Debian: `.deb`
- Generic Linux: `.tar.gz`
- Windows: `.msi` and `.zip`
- macOS: `.pkg`

macOS and Windows artifacts are not signed yet.

## Hardware Notes

The command table follows the MSI HID protocol used by `couriersud/msigd`. DDC/CI is not used.

If MD342CQPW command values differ from the current MD342-family assumptions, capture the official MSI app HID traffic and update the setting table in `crates/pip-switch-core/src/settings.rs`.
