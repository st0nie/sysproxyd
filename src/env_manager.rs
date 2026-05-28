use crate::config::{ProxyConfig, ProxyMode};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::env;
use zbus::blocking::Connection;

pub struct EnvManager;

const ENV_HTTP_PROXY: &str = "http_proxy";
const ENV_HTTPS_PROXY: &str = "https_proxy";
const ENV_FTP_PROXY: &str = "ftp_proxy";
const ENV_ALL_PROXY: &str = "all_proxy";
const ENV_NO_PROXY: &str = "no_proxy";

impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvManager {
    pub fn new() -> Self {
        Self
    }

    pub fn apply(&self, config: &ProxyConfig) {
        info!("Applying proxy config: mode={}", config.mode);

        self.clear_all_envs();
        self.clear_systemd_dbus_envs();

        match config.mode {
            ProxyMode::None => {
                info!("Proxy mode is 'none', all proxy environment variables cleared");
            }
            ProxyMode::Manual => {
                info!("Proxy mode is 'manual', applying manual proxy settings");
                self.apply_manual(config);
            }
            ProxyMode::Auto => {
                info!(
                    "Proxy mode is 'auto', proxy configuration read but no standard env vars are set"
                );
            }
        }
    }

    fn apply_manual(&self, config: &ProxyConfig) {
        if let Some(ref s) = config.http {
            let url = s.to_proxy_url("http");
            info!("Setting HTTP proxy: {}", url);
            self.set_env(ENV_HTTP_PROXY, &url);
        } else {
            info!("No HTTP proxy configured");
        }
        if let Some(ref s) = config.https {
            let url = s.to_proxy_url("http");
            info!("Setting HTTPS proxy: {}", url);
            self.set_env(ENV_HTTPS_PROXY, &url);
        } else {
            info!("No HTTPS proxy configured");
        }
        if let Some(ref s) = config.ftp {
            let url = s.to_proxy_url("http");
            info!("Setting FTP proxy: {}", url);
            self.set_env(ENV_FTP_PROXY, &url);
        } else {
            info!("No FTP proxy configured");
        }
        if let Some(ref s) = config.socks {
            let url = s.to_proxy_url("socks5");
            info!("Setting SOCKS proxy: {}", url);
            self.set_env(ENV_ALL_PROXY, &url);
        } else {
            info!("No SOCKS proxy configured");
        }
        if !config.no_proxy.is_empty() {
            let no_proxy = config.no_proxy.join(",");
            info!("Setting no_proxy: {}", no_proxy);
            self.set_env(ENV_NO_PROXY, &no_proxy);
        } else {
            info!("No no_proxy list configured");
        }
    }

    fn set_env(&self, key: &str, value: &str) {
        unsafe { env::set_var(key, value) };
        debug!("Set env: {}={}", key, value);

        if let Err(e) = self.set_systemd_env(key, value) {
            warn!("Failed to set systemd env {}: {}", key, e);
        }
        if let Err(e) = self.set_dbus_env(key, value) {
            warn!("Failed to set dbus env {}: {}", key, e);
        }
    }

    /// 清除所有代理环境变量
    fn clear_all_envs(&self) {
        for key in [
            ENV_HTTP_PROXY,
            ENV_HTTPS_PROXY,
            ENV_FTP_PROXY,
            ENV_ALL_PROXY,
            ENV_NO_PROXY,
        ] {
            unsafe { env::remove_var(key) };
        }
    }

    /// 清除 systemd 和 dbus 中的代理环境变量
    fn clear_systemd_dbus_envs(&self) {
        for key in [
            ENV_HTTP_PROXY,
            ENV_HTTPS_PROXY,
            ENV_FTP_PROXY,
            ENV_ALL_PROXY,
            ENV_NO_PROXY,
        ] {
            if let Err(e) = self.unset_systemd_env(key) {
                warn!("Failed to unset systemd env {}: {}", key, e);
            }
            if let Err(e) = self.unset_dbus_env(key) {
                warn!("Failed to unset dbus env {}: {}", key, e);
            }
        }
    }

    /// 通过 systemd D-Bus 设置环境变量
    fn set_systemd_env(&self, key: &str, value: &str) -> zbus::Result<()> {
        let conn = Connection::session()?;
        let proxy = conn.call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "SetEnvironment",
            &(vec![format!("{}={}", key, value)],),
        )?;
        debug!("Set systemd env: {}={}", key, value);
        drop(proxy);
        Ok(())
    }

    /// 通过 systemd D-Bus 取消设置环境变量
    fn unset_systemd_env(&self, key: &str) -> zbus::Result<()> {
        let conn = Connection::session()?;
        let proxy = conn.call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "UnsetEnvironment",
            &(vec![key.to_string()],),
        )?;
        debug!("Unset systemd env: {}", key);
        drop(proxy);
        Ok(())
    }

    /// 通过 D-Bus daemon 设置激活环境变量
    fn set_dbus_env(&self, key: &str, value: &str) -> zbus::Result<()> {
        let conn = Connection::session()?;
        let mut env_map = HashMap::new();
        env_map.insert(key.to_string(), value.to_string());
        let proxy = conn.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "UpdateActivationEnvironment",
            &(env_map),
        )?;
        debug!("Set dbus env: {}={}", key, value);
        drop(proxy);
        Ok(())
    }

    fn unset_dbus_env(&self, key: &str) -> zbus::Result<()> {
        let conn = Connection::session()?;
        let mut env_map = HashMap::new();
        env_map.insert(key.to_string(), String::new());
        let proxy = conn.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "UpdateActivationEnvironment",
            &(env_map),
        )?;
        debug!("Unset dbus env: {}", key);
        drop(proxy);
        Ok(())
    }
}
