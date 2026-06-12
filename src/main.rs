use anyhow::{Context, Result};
use clap::Parser;
use glib::MainLoop;
use log::{info, warn};

use sysproxyd::env_manager::EnvManager;
use sysproxyd::gsettings::{self, GSettingsError};

/// System proxy daemon — syncs GNOME/GSettings proxy config to shell env vars and systemd.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Disable timestamps in log output (useful for journald/syslog)
    #[arg(long)]
    no_timestamp: bool,

    /// Use socks5h scheme for `all_proxy` instead of socks5
    #[arg(long)]
    use_socks5h: bool,

    /// Apply the current configuration once and exit
    #[arg(long)]
    once: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    if cli.no_timestamp {
        builder.format_timestamp(None);
    }

    builder.init();
    info!("sysproxyd starting...");

    let env_manager = EnvManager::new(cli.use_socks5h);

    apply_current_config(&env_manager).context("failed to apply initial proxy config")?;

    if cli.once {
        info!("sysproxyd applied config once, exiting");
        return Ok(());
    }

    let env_manager_clone = env_manager.clone();
    let _watcher = gsettings::watch(move || {
        info!("GSettings proxy config changed, reapplying...");
        if let Err(e) = apply_current_config(&env_manager_clone) {
            warn!("{e}");
        }
    });

    let main_loop = MainLoop::new(None, false);
    main_loop.run();
    Ok(())
}

fn apply_current_config(env_manager: &EnvManager) -> Result<()> {
    match gsettings::read_config() {
        Ok(config) => env_manager
            .try_apply(&config)
            .context("failed to propagate proxy envs"),
        Err(GSettingsError::SchemaNotAvailable) => {
            warn!("GSettings proxy schema not available; no config applied");
            Ok(())
        }
    }
}
