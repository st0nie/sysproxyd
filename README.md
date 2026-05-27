# sysproxyd

A lightweight system proxy daemon that automatically synchronizes desktop proxy configuration to shell environment variables, systemd, and D-Bus activation environments.

## Features

- **Real-time sync** — Watches desktop proxy settings (GNOME/GSettings) and applies changes immediately
- **Multi-protocol support** — HTTP, HTTPS, FTP, and SOCKS proxies
- **Authentication** — Supports proxy authentication with URL-safe encoding
- **System-wide propagation** — Updates both process environment and systemd/D-Bus activation env
- **Auto proxy** — Supports PAC (Proxy Auto-Config) URL mode

## Inspiration

This project is inspired by the system proxy implementation in **DDE (Deepin Desktop Environment)**.

DDE's `go-lib/proxy` module pioneered the approach of:

1. Listening to gsettings proxy configuration changes
2. Converting proxy settings to standard environment variables (`http_proxy`, `https_proxy`, `all_proxy`, etc.)
3. Propagating the configuration to the entire user session via D-Bus calls to `org.freedesktop.systemd1.Manager.SetEnvironment` and `org.freedesktop.DBus.UpdateActivationEnvironment`

`sysproxyd` re-implements this mechanism in Rust. While it reads proxy settings from GNOME/GSettings, the standard environment variables it sets (`http_proxy`, `https_proxy`, `all_proxy`, `no_proxy`) are respected by most Linux applications — including those running on **KDE Plasma**, XFCE, and other desktop environments.

## Requirements

- Rust toolchain (edition 2024)
- GNOME desktop environment (for GSettings `org.gnome.system.proxy` schema; KDE and other desktops benefit from the propagated environment variables)
- D-Bus session bus
- `libgio` development headers (for `gio` crate build dependencies)

## Installation

### From source

```bash
git clone https://github.com/st0nie/sysproxyd
cd sysproxyd
./install.sh
```

The install script compiles and installs the binary via `cargo`, then copies the systemd user service file to `~/.config/systemd/user/`.

### Enable and start

```bash
systemctl --user enable sysproxyd
systemctl --user start sysproxyd
systemctl --user status sysproxyd
```

## Manual build

```bash
cargo build --release
```

The binary will be available at `target/release/sysproxyd`.

## Testing

```bash
cargo test
```

> **Note:** Integration tests manipulate process environment variables. They rely on `serial_test` to prevent race conditions. Tests that require GSettings will gracefully skip when GNOME is not available.

## Architecture

| File | Purpose |
|------|---------|
| `src/main.rs` | Daemon entrypoint — glib MainLoop, wires watcher to env manager |
| `src/gsettings.rs` | Reads `org.gnome.system.proxy` via `gio::Settings`; watches for changes |
| `src/env_manager.rs` | Applies `ProxyConfig` to env vars + systemd/dbus activation env |
| `src/config.rs` | Data structures: `ProxyMode`, `ProxyServer`, `ProxyAuth`, `ProxyConfig` |

## Roadmap / TODO

- [ ] **KDE kioslave sync** — Bidirectional synchronization with KDE Plasma's `kioslaverc` (`~/.config/kioslaverc`) so that KDE System Settings proxy changes are also picked up and applied as environment variables, and vice versa.

## License

MIT
