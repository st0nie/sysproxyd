use std::rc::Rc;

use clap::Parser;
use glib::MainLoop;
use log::info;

use sysproxyd::env_manager::EnvManager;
use sysproxyd::gsettings;

/// System proxy daemon — syncs GNOME/GSettings proxy config to shell env vars and systemd.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Disable timestamps in log output (useful for journald/syslog)
    #[arg(long)]
    no_timestamp: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    if cli.no_timestamp {
        builder.format_timestamp(None);
    }

    builder.init();
    info!("sysproxyd starting...");

    let env_manager = Rc::new(EnvManager::new());

    let initial_config = gsettings::read_config();
    if let Some(ref config) = initial_config {
        env_manager.apply(config);
    }

    let env_manager_clone = env_manager.clone();
    let _watcher = gsettings::watch(move || {
        info!("GSettings proxy config changed, reapplying...");
        if let Some(config) = gsettings::read_config() {
            env_manager_clone.apply(&config);
        }
    });

    let main_loop = MainLoop::new(None, false);
    main_loop.run();
}
