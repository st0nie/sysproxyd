use percent_encoding::utf8_percent_encode;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyMode {
    None,
    Manual,
    Auto,
}

impl ProxyMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "manual" => ProxyMode::Manual,
            "auto" => ProxyMode::Auto,
            _ => ProxyMode::None,
        }
    }
}

impl fmt::Display for ProxyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyMode::None => write!(f, "none"),
            ProxyMode::Manual => write!(f, "manual"),
            ProxyMode::Auto => write!(f, "auto"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyAuth {
    pub username: String,
    password: String,
}

impl ProxyAuth {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn as_url_prefix(&self) -> String {
        format!(
            "{}:{}@",
            percent_encode(&self.username),
            percent_encode(&self.password)
        )
    }
}

impl fmt::Display for ProxyAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:***", self.username)
    }
}

#[derive(Debug, Clone)]
pub struct ProxyServer {
    pub host: String,
    pub port: u16,
    pub auth: Option<ProxyAuth>,
}

impl ProxyServer {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            auth: None,
        }
    }

    pub fn with_auth(mut self, auth: ProxyAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn to_proxy_url(&self, scheme: &str) -> String {
        let auth = self
            .auth
            .as_ref()
            .map(|a| a.as_url_prefix())
            .unwrap_or_default();
        format!("{}://{}{}:{}", scheme, auth, self.host, self.port)
    }
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub mode: ProxyMode,
    pub http: Option<ProxyServer>,
    pub https: Option<ProxyServer>,
    pub ftp: Option<ProxyServer>,
    pub socks: Option<ProxyServer>,
    pub auto_url: Option<String>,
    pub no_proxy: Vec<String>,
}

impl ProxyConfig {
    pub fn new() -> Self {
        Self {
            mode: ProxyMode::None,
            http: None,
            https: None,
            ftp: None,
            socks: None,
            auto_url: None,
            no_proxy: Vec::new(),
        }
    }
}

const URL_ENCODE_SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

fn percent_encode(s: &str) -> String {
    utf8_percent_encode(s, URL_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_mode_from_str() {
        assert_eq!(ProxyMode::from_str("manual"), ProxyMode::Manual);
        assert_eq!(ProxyMode::from_str("auto"), ProxyMode::Auto);
        assert_eq!(ProxyMode::from_str("none"), ProxyMode::None);
        assert_eq!(ProxyMode::from_str(""), ProxyMode::None);
        assert_eq!(ProxyMode::from_str("unknown"), ProxyMode::None);
    }

    #[test]
    fn test_proxy_mode_display() {
        assert_eq!(ProxyMode::None.to_string(), "none");
        assert_eq!(ProxyMode::Manual.to_string(), "manual");
        assert_eq!(ProxyMode::Auto.to_string(), "auto");
    }

    #[test]
    fn test_proxy_auth_new() {
        let auth = ProxyAuth::new("user", "pass");
        assert_eq!(auth.username, "user");
        assert_eq!(auth.password, "pass");
    }

    #[test]
    fn test_proxy_auth_as_url_prefix() {
        let auth = ProxyAuth::new("user", "pass");
        assert_eq!(auth.as_url_prefix(), "user:pass@");
    }

    #[test]
    fn test_proxy_auth_url_encoding() {
        let auth = ProxyAuth::new("user@domain", "p@ss:w#rd");
        assert_eq!(auth.as_url_prefix(), "user%40domain:p%40ss%3Aw%23rd@");
    }

    #[test]
    fn test_proxy_auth_display() {
        let auth = ProxyAuth::new("admin", "secret");
        assert_eq!(auth.to_string(), "admin:***");
    }

    #[test]
    fn test_proxy_server_new() {
        let server = ProxyServer::new("proxy.example.com", 8080);
        assert_eq!(server.host, "proxy.example.com");
        assert_eq!(server.port, 8080);
        assert!(server.auth.is_none());
    }

    #[test]
    fn test_proxy_server_with_auth() {
        let server =
            ProxyServer::new("proxy.example.com", 8080).with_auth(ProxyAuth::new("user", "pass"));
        assert!(server.auth.is_some());
        assert_eq!(server.auth.as_ref().unwrap().username, "user");
    }

    #[test]
    fn test_proxy_server_to_proxy_url_without_auth() {
        let server = ProxyServer::new("proxy.example.com", 8080);
        assert_eq!(server.to_proxy_url("http"), "http://proxy.example.com:8080");
    }

    #[test]
    fn test_proxy_server_to_proxy_url_with_auth() {
        let server =
            ProxyServer::new("proxy.example.com", 8080).with_auth(ProxyAuth::new("user", "pass"));
        assert_eq!(
            server.to_proxy_url("http"),
            "http://user:pass@proxy.example.com:8080"
        );
    }

    #[test]
    fn test_proxy_server_to_proxy_url_socks5() {
        let server = ProxyServer::new("socks.example.com", 1080);
        assert_eq!(
            server.to_proxy_url("socks5"),
            "socks5://socks.example.com:1080"
        );
    }

    #[test]
    fn test_proxy_config_new() {
        let config = ProxyConfig::new();
        assert_eq!(config.mode, ProxyMode::None);
        assert!(config.http.is_none());
        assert!(config.https.is_none());
        assert!(config.ftp.is_none());
        assert!(config.socks.is_none());
        assert!(config.auto_url.is_none());
        assert!(config.no_proxy.is_empty());
    }
}
