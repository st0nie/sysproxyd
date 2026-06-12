use crate::config::{ProxyConfig, ProxyMode};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::env;
use zbus::blocking::Connection;

#[derive(Debug, Clone)]
pub struct EnvManager {
    use_socks5h: bool,
}

const ENV_HTTP_PROXY: &str = "http_proxy";
const ENV_HTTPS_PROXY: &str = "https_proxy";
const ENV_FTP_PROXY: &str = "ftp_proxy";
const ENV_ALL_PROXY: &str = "all_proxy";
const ENV_NO_PROXY: &str = "no_proxy";

const ALL_PROXY_KEYS: [&str; 5] = [
    ENV_HTTP_PROXY,
    ENV_HTTPS_PROXY,
    ENV_FTP_PROXY,
    ENV_ALL_PROXY,
    ENV_NO_PROXY,
];

impl Default for EnvManager {
    fn default() -> Self {
        Self::new(false)
    }
}

impl EnvManager {
    #[must_use]
    pub fn new(use_socks5h: bool) -> Self {
        Self { use_socks5h }
    }

    pub fn apply(&self, config: &ProxyConfig) {
        info!("Applying proxy config: mode={}", config.mode);

        Self::clear_all_envs();

        let envs = match config.mode {
            ProxyMode::None => {
                info!("Proxy mode is 'none', all proxy environment variables cleared");
                Vec::new()
            }
            ProxyMode::Manual => {
                info!("Proxy mode is 'manual', applying manual proxy settings");
                self.build_manual_envs(config)
            }
            ProxyMode::Auto => {
                info!(
                    "Proxy mode is 'auto', proxy configuration read but no standard env vars are set"
                );
                Vec::new()
            }
        };

        for (key, value) in &envs {
            unsafe { env::set_var(key, value) };
            debug!("Set env: {key}={value}");
        }

        if let Err(e) = Self::propagate_all(&envs) {
            warn!("Failed to propagate proxy envs to systemd/dbus: {e}");
        }
    }

    fn build_manual_envs(&self, config: &ProxyConfig) -> Vec<(&'static str, String)> {
        let mut envs = Vec::with_capacity(5);

        if let Some(server) = &config.http {
            let url = server.to_proxy_url("http");
            info!("Setting HTTP proxy: {url}");
            envs.push((ENV_HTTP_PROXY, url));
        } else {
            info!("No HTTP proxy configured");
        }

        if let Some(server) = &config.https {
            let url = server.to_proxy_url("http");
            info!("Setting HTTPS proxy: {url}");
            envs.push((ENV_HTTPS_PROXY, url));
        } else {
            info!("No HTTPS proxy configured");
        }

        if let Some(server) = &config.ftp {
            let url = server.to_proxy_url("http");
            info!("Setting FTP proxy: {url}");
            envs.push((ENV_FTP_PROXY, url));
        } else {
            info!("No FTP proxy configured");
        }

        if let Some(server) = &config.socks {
            let scheme = if self.use_socks5h {
                "socks5h"
            } else {
                "socks5"
            };
            let url = server.to_proxy_url(scheme);
            info!("Setting SOCKS proxy: {url}");
            envs.push((ENV_ALL_PROXY, url));
        } else {
            info!("No SOCKS proxy configured");
        }

        if config.no_proxy.is_empty() {
            info!("No no_proxy list configured");
        } else {
            let no_proxy = config.no_proxy.join(",");
            info!("Setting no_proxy: {no_proxy}");
            envs.push((ENV_NO_PROXY, no_proxy));
        }

        envs
    }

    fn clear_all_envs() {
        for key in ALL_PROXY_KEYS {
            unsafe { env::remove_var(key) };
        }
    }

    /// Propagate the desired proxy environment to systemd and D-Bus in batched calls.
    fn propagate_all(envs: &[(&'static str, String)]) -> zbus::Result<()> {
        let conn = Connection::session()?;

        let env_map: HashMap<&str, &str> = envs
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect();

        let mut systemd_assignments = Vec::with_capacity(envs.len());
        let mut systemd_removals = Vec::with_capacity(ALL_PROXY_KEYS.len());
        let mut dbus_envs = HashMap::with_capacity(ALL_PROXY_KEYS.len());

        for key in ALL_PROXY_KEYS {
            if let Some(value) = env_map.get(key) {
                systemd_assignments.push(format!("{key}={value}"));
                dbus_envs.insert(key.to_string(), value.to_string());
            } else {
                systemd_removals.push(key.to_string());
                dbus_envs.insert(key.to_string(), String::new());
            }
        }

        if !systemd_removals.is_empty() {
            conn.call_method(
                Some("org.freedesktop.systemd1"),
                "/org/freedesktop/systemd1",
                Some("org.freedesktop.systemd1.Manager"),
                "UnsetEnvironment",
                &(systemd_removals.clone(),),
            )?;
            debug!("Unset systemd envs: {systemd_removals:?}");
        }

        if !systemd_assignments.is_empty() {
            conn.call_method(
                Some("org.freedesktop.systemd1"),
                "/org/freedesktop/systemd1",
                Some("org.freedesktop.systemd1.Manager"),
                "SetEnvironment",
                &(systemd_assignments.clone(),),
            )?;
            debug!("Set systemd envs: {systemd_assignments:?}");
        }

        conn.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "UpdateActivationEnvironment",
            &(dbus_envs,),
        )?;
        debug!("Updated dbus activation envs");

        Ok(())
    }
}
