use crate::config::{ProxyAuth, ProxyConfig, ProxyMode, ProxyServer};
use gio::Settings;
use gio::prelude::*;
use glib::SignalHandlerId;
use log::{info, warn};

const SCHEMA: &str = "org.gnome.system.proxy";
const KEY_MODE: &str = "mode";
const KEY_AUTOCONFIG_URL: &str = "autoconfig-url";
const KEY_IGNORE_HOSTS: &str = "ignore-hosts";

const CHILD_HTTP: &str = "http";
const CHILD_HTTPS: &str = "https";
const CHILD_FTP: &str = "ftp";
const CHILD_SOCKS: &str = "socks";

const KEY_HOST: &str = "host";
const KEY_PORT: &str = "port";
const KEY_USE_AUTH: &str = "use-authentication";
const KEY_AUTH_USER: &str = "authentication-user";
const KEY_AUTH_PASS: &str = "authentication-password";

#[must_use]
pub fn is_available() -> bool {
    gio::SettingsSchemaSource::default()
        .and_then(|source| source.lookup(SCHEMA, true))
        .is_some()
}

#[must_use]
pub fn read_config() -> Option<ProxyConfig> {
    if !is_available() {
        warn!("GSettings schema '{SCHEMA}' not found");
        return None;
    }

    let settings = Settings::new(SCHEMA);
    let mut config = ProxyConfig::new();

    let mode = settings.string(KEY_MODE);
    config.mode = mode.parse().unwrap_or_else(|_| {
        warn!("Unknown proxy mode '{mode}', defaulting to none");
        ProxyMode::None
    });
    info!("Read proxy mode from GSettings: {}", config.mode);

    match config.mode {
        ProxyMode::Manual => {
            let http = settings.child(CHILD_HTTP);
            let https = settings.child(CHILD_HTTPS);
            let ftp = settings.child(CHILD_FTP);
            let socks = settings.child(CHILD_SOCKS);

            config.http = read_http_server(&http);
            config.https = read_server(&https);
            config.ftp = read_server(&ftp);
            config.socks = read_server(&socks);
            config.no_proxy = read_no_proxy(&settings);
        }
        ProxyMode::Auto => {
            config.auto_url = Some(settings.string(KEY_AUTOCONFIG_URL).to_string());
        }
        ProxyMode::None => {}
    }

    Some(config)
}

pub struct Watcher {
    _settings: Settings,
    _http: Settings,
    _https: Settings,
    _ftp: Settings,
    _socks: Settings,
    _ids: Vec<SignalHandlerId>,
}

pub fn watch<F>(callback: F) -> Option<Watcher>
where
    F: Fn() + Clone + 'static,
{
    if !is_available() {
        return None;
    }

    let settings = Settings::new(SCHEMA);
    let http = settings.child(CHILD_HTTP);
    let https = settings.child(CHILD_HTTPS);
    let ftp = settings.child(CHILD_FTP);
    let socks = settings.child(CHILD_SOCKS);

    let mut ids = Vec::new();

    let cb = callback.clone();
    ids.push(settings.connect_changed(None, move |_, _| cb()));

    let cb = callback.clone();
    ids.push(http.connect_changed(None, move |_, _| cb()));

    let cb = callback.clone();
    ids.push(https.connect_changed(None, move |_, _| cb()));

    let cb = callback.clone();
    ids.push(ftp.connect_changed(None, move |_, _| cb()));

    let cb = callback;
    ids.push(socks.connect_changed(None, move |_, _| cb()));

    Some(Watcher {
        _settings: settings,
        _http: http,
        _https: https,
        _ftp: ftp,
        _socks: socks,
        _ids: ids,
    })
}

fn read_server(child: &Settings) -> Option<ProxyServer> {
    let host = child.string(KEY_HOST);
    if host.is_empty() {
        return None;
    }

    let port = u16::try_from(child.int(KEY_PORT)).ok()?;
    Some(ProxyServer::new(host.to_string(), port))
}

fn read_http_server(child: &Settings) -> Option<ProxyServer> {
    let mut server = read_server(child)?;

    if child.boolean(KEY_USE_AUTH) {
        let user = child.string(KEY_AUTH_USER);
        let pass = child.string(KEY_AUTH_PASS);
        if !user.is_empty() {
            server = server.with_auth(ProxyAuth::new(user.to_string(), pass.to_string()));
        }
    }

    Some(server)
}

fn read_no_proxy(settings: &Settings) -> Vec<String> {
    settings
        .strv(KEY_IGNORE_HOSTS)
        .iter()
        .map(std::string::ToString::to_string)
        .collect()
}
