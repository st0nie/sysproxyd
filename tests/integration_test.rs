use serial_test::serial;
use std::env;
use std::str::FromStr as _;
use sysproxyd::config::{ProxyAuth, ProxyConfig, ProxyMode, ProxyServer};
use sysproxyd::env_manager::EnvManager;
use sysproxyd::gsettings;

fn set_env(key: &str, value: &str) {
    unsafe { env::set_var(key, value) };
}

/// 测试 ProxyMode 的字符串解析和显示转换
#[test]
fn test_proxy_mode_roundtrip() {
    assert_eq!(ProxyMode::from_str("manual"), Ok(ProxyMode::Manual));
    assert_eq!(ProxyMode::from_str("auto"), Ok(ProxyMode::Auto));
    assert_eq!(ProxyMode::from_str("none"), Ok(ProxyMode::None));
    assert_eq!(ProxyMode::from_str(""), Ok(ProxyMode::None));
    assert_eq!(ProxyMode::from_str("unknown"), Ok(ProxyMode::None));

    assert_eq!(ProxyMode::None.to_string(), "none");
    assert_eq!(ProxyMode::Manual.to_string(), "manual");
    assert_eq!(ProxyMode::Auto.to_string(), "auto");
}

/// 测试 ProxyAuth 的创建和 URL 前缀生成
#[test]
fn test_proxy_auth_url_prefix() {
    let auth = ProxyAuth::new("user", "pass");
    assert_eq!(auth.as_url_prefix(), "user:pass@");
    assert_eq!(auth.to_string(), "user:***");

    let auth_special = ProxyAuth::new("user@domain", "p@ss:w#rd");
    assert_eq!(
        auth_special.as_url_prefix(),
        "user%40domain:p%40ss%3Aw%23rd@"
    );
}

/// 测试 ProxyServer 的 URL 生成（无认证）
#[test]
fn test_proxy_server_url_without_auth() {
    let server = ProxyServer::new("proxy.example.com", 8080);
    assert_eq!(server.to_proxy_url("http"), "http://proxy.example.com:8080");
    assert_eq!(
        server.to_proxy_url("https"),
        "https://proxy.example.com:8080"
    );
    assert_eq!(server.to_proxy_url("ftp"), "ftp://proxy.example.com:8080");
    assert_eq!(
        server.to_proxy_url("socks5"),
        "socks5://proxy.example.com:8080"
    );
}

/// 测试 ProxyServer 的 URL 生成（有认证）
#[test]
fn test_proxy_server_url_with_auth() {
    let server =
        ProxyServer::new("proxy.example.com", 8080).with_auth(ProxyAuth::new("admin", "secret123"));
    assert_eq!(
        server.to_proxy_url("http"),
        "http://admin:secret123@proxy.example.com:8080"
    );
}

/// 测试 ProxyConfig 的默认状态
#[test]
fn test_proxy_config_default() {
    let config = ProxyConfig::new();
    assert_eq!(config.mode, ProxyMode::None);
    assert!(config.http.is_none());
    assert!(config.https.is_none());
    assert!(config.ftp.is_none());
    assert!(config.socks.is_none());
    assert!(config.auto_url.is_none());
    assert!(config.no_proxy.is_empty());
}

/// 测试完整手动代理配置的场景
#[test]
fn test_full_manual_proxy_config() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Manual;
    config.http = Some(ProxyServer::new("http-proxy.local", 3128));
    config.https = Some(ProxyServer::new("https-proxy.local", 3129));
    config.ftp = Some(ProxyServer::new("ftp-proxy.local", 3130));
    config.socks = Some(ProxyServer::new("socks-proxy.local", 1080));
    config.no_proxy = vec!["localhost".to_string(), "127.0.0.1".to_string()];

    assert_eq!(config.mode, ProxyMode::Manual);
    assert_eq!(config.http.as_ref().unwrap().host, "http-proxy.local");
    assert_eq!(config.https.as_ref().unwrap().port, 3129);
    assert_eq!(config.no_proxy.len(), 2);
}

/// 测试自动代理配置
#[test]
fn test_auto_proxy_config() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Auto;
    config.auto_url = Some("http://proxy.pac".to_string());

    assert_eq!(config.mode, ProxyMode::Auto);
    assert_eq!(config.auto_url.as_ref().unwrap(), "http://proxy.pac");
}

/// 测试 EnvManager 应用 None 模式时清除所有环境变量
#[test]
#[serial]
fn test_env_manager_apply_none_clears_envs() {
    // 预先设置一些代理环境变量
    set_env("http_proxy", "http://old:8080");
    set_env("https_proxy", "http://old:8080");
    set_env("ftp_proxy", "http://old:8080");
    set_env("all_proxy", "socks5://old:1080");
    set_env("no_proxy", "old.local");

    let manager = EnvManager::new();
    let config = ProxyConfig::new();
    manager.apply(&config);

    assert!(env::var("http_proxy").is_err());
    assert!(env::var("https_proxy").is_err());
    assert!(env::var("ftp_proxy").is_err());
    assert!(env::var("all_proxy").is_err());
    assert!(env::var("no_proxy").is_err());
}

/// 测试 EnvManager 应用手动代理配置
#[test]
#[serial]
fn test_env_manager_apply_manual() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Manual;
    config.http = Some(ProxyServer::new("http-proxy.local", 3128));
    config.https =
        Some(ProxyServer::new("https-proxy.local", 3129).with_auth(ProxyAuth::new("user", "pass")));
    config.no_proxy = vec!["localhost".to_string(), "127.0.0.1".to_string()];

    let manager = EnvManager::new();
    manager.apply(&config);

    assert_eq!(
        env::var("http_proxy").unwrap(),
        "http://http-proxy.local:3128"
    );
    assert_eq!(
        env::var("https_proxy").unwrap(),
        "http://user:pass@https-proxy.local:3129"
    );
    assert_eq!(env::var("no_proxy").unwrap(), "localhost,127.0.0.1");

    assert!(env::var("ftp_proxy").is_err());
    assert!(env::var("all_proxy").is_err());
}

/// 测试 EnvManager 应用自动代理配置
#[test]
#[serial]
fn test_env_manager_apply_auto() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Auto;
    config.auto_url = Some("http://proxy.pac".to_string());

    let manager = EnvManager::new();
    manager.apply(&config);

    assert!(env::var("http_proxy").is_err());
    assert!(env::var("https_proxy").is_err());
    assert!(env::var("ftp_proxy").is_err());
    assert!(env::var("all_proxy").is_err());
    assert!(env::var("no_proxy").is_err());
}

/// 测试 EnvManager 从手动模式切换到 None 模式时正确清除
#[test]
#[serial]
fn test_env_manager_switch_from_manual_to_none() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Manual;
    config.http = Some(ProxyServer::new("proxy.local", 8080));

    let manager = EnvManager::new();
    manager.apply(&config);
    assert_eq!(env::var("http_proxy").unwrap(), "http://proxy.local:8080");

    let none_config = ProxyConfig::new();
    manager.apply(&none_config);
    assert!(env::var("http_proxy").is_err());
}

/// 测试 EnvManager 覆盖旧配置
#[test]
#[serial]
fn test_env_manager_overwrite_config() {
    let mut config1 = ProxyConfig::new();
    config1.mode = ProxyMode::Manual;
    config1.http = Some(ProxyServer::new("old-proxy", 8080));

    let manager = EnvManager::new();
    manager.apply(&config1);
    assert_eq!(env::var("http_proxy").unwrap(), "http://old-proxy:8080");

    let mut config2 = ProxyConfig::new();
    config2.mode = ProxyMode::Manual;
    config2.http = Some(ProxyServer::new("new-proxy", 9090));
    manager.apply(&config2);
    assert_eq!(env::var("http_proxy").unwrap(), "http://new-proxy:9090");
}

/// 测试 GSettings 可用性检测（不依赖实际 GNOME 环境）
#[test]
fn test_gsettings_availability() {
    let _available = gsettings::is_available();
    if !_available {
        assert!(gsettings::read_config().is_none());
    }
}

/// 测试 SOCKS 代理配置
#[test]
#[serial]
fn test_env_manager_apply_socks() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Manual;
    config.socks = Some(ProxyServer::new("socks.local", 1080));

    let manager = EnvManager::new();
    manager.apply(&config);

    assert_eq!(env::var("all_proxy").unwrap(), "socks5://socks.local:1080");
}

/// 测试带认证的 SOCKS 代理
#[test]
#[serial]
fn test_env_manager_apply_socks_with_auth() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Manual;
    config.socks =
        Some(ProxyServer::new("socks.local", 1080).with_auth(ProxyAuth::new("user", "pass")));

    let manager = EnvManager::new();
    manager.apply(&config);

    assert_eq!(
        env::var("all_proxy").unwrap(),
        "socks5://user:pass@socks.local:1080"
    );
}

/// 测试 FTP 代理配置
#[test]
#[serial]
fn test_env_manager_apply_ftp() {
    let mut config = ProxyConfig::new();
    config.mode = ProxyMode::Manual;
    config.ftp = Some(ProxyServer::new("ftp-proxy.local", 2121));

    let manager = EnvManager::new();
    manager.apply(&config);

    assert_eq!(
        env::var("ftp_proxy").unwrap(),
        "http://ftp-proxy.local:2121"
    );
}
