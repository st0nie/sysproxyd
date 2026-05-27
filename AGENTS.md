# sysproxyd — Agent Guide

## Project

Rust daemon that syncs GNOME/GSettings proxy configuration to shell environment variables and systemd/D-Bus activation environments.

## Build & Test

```bash
cargo build          # debug
cargo build --release
cargo test           # runs unit + integration tests
cargo run            # runs the daemon (needs GNOME session for full function)
```

## Critical Testing Notes

- **Integration tests mutate process environment variables.** Every `EnvManager` test must be annotated with `#[serial]` from `serial_test`. Without it, tests race and flake.
- `gsettings::is_available()` returns `false` outside a GNOME session. Tests that call `read_config()` or `watch()` outside GNOME get `None` — this is expected, not a failure.

## Runtime Requirements

- GNOME/GSettings schema `org.gnome.system.proxy` must be installed.
- D-Bus session bus must be available (used for `systemd` and `org.freedesktop.DBus` env propagation).
- The binary is typically run as a systemd user service (`install/sysproxyd.service`).

## Architecture

```
src/main.rs        glib MainLoop, wires gsettings watcher → env_manager
src/gsettings.rs   Reads org.gnome.system.proxy via gio::Settings; watches changes
src/env_manager.rs Applies ProxyConfig to env vars + systemd/dbus activation env
src/config.rs      ProxyMode, ProxyServer, ProxyAuth, ProxyConfig (no I/O)
```

## Conventions

- Rust edition **2024**.
- `unsafe { env::set_var(...) }` and `env::remove_var(...)` are intentional — these APIs are unsafe in Rust 2024. Do not refactor them away.
- Some inline comments are in Chinese; preserve them when editing nearby code.
- No formatter or linter config present — follow `cargo fmt` / `cargo clippy` defaults.

## Installation

```bash
./install.sh   # cargo install --path . + copies systemd user service
```

After install, enable/start with:
```bash
systemctl --user enable sysproxyd
systemctl --user start sysproxyd
```
