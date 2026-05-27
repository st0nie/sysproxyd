use std::sync::Arc;

use glib::MainLoop;
use log::info;

use sysproxyd::env_manager::EnvManager;
use sysproxyd::gsettings;

fn main() {
    let no_timestamp = std::env::args().any(|arg| arg == "--no-timestamp");

    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(log::LevelFilter::Info);

    if no_timestamp {
        builder.format_timestamp(None);
    }

    builder.init();
    info!("sysproxyd starting...");

    let env_manager = Arc::new(EnvManager::new());

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
