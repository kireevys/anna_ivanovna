use std::path::PathBuf;

use ai_app::config;
use tracing::info;

const DEFAULT_HOME: &str = ".buh";
const LOG_FILE: &str = "anna_ivanovna.log";

#[derive(Default)]
pub struct ConfigOverrides {
    pub host: Option<String>,
    pub port: Option<u16>,
}

/// Resolve buh_home, init logging, load config with overrides.
pub fn init(
    buh_home: Option<PathBuf>,
    overrides: ConfigOverrides,
) -> Result<config::Config, Box<dyn std::error::Error>> {
    let home = resolve_home(buh_home)?;

    crate::infra::logging::init(&home, LOG_FILE)?;

    let config_path = home.join("config.json");
    let mut cfg: config::Config = config::read_json(&config_path)?;

    if let Some(host) = overrides.host {
        cfg.server.host = host;
    }
    if let Some(port) = overrides.port {
        cfg.server.port = port;
    }

    Ok(cfg)
}

fn resolve_home(buh_home: Option<PathBuf>) -> Result<PathBuf, config::Error> {
    if let Some(path) = buh_home {
        info!("buh_home from CLI/env: {}", path.display());
        return Ok(path);
    }

    let default = dirs::home_dir()
        .map(|h| h.join(DEFAULT_HOME))
        .ok_or(config::Error::HomeDirNotFound)?;
    info!("buh_home default: {}", default.display());
    Ok(default)
}
