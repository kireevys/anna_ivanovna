use thiserror::Error;
use tracing::info;

const DEFAULT_HOME: &str = ".buh";
const ENV_BUH_HOME: &str = "BUH_HOME";

#[derive(Debug, Error)]
pub enum Error {
    #[error("config not found")]
    NoConfig,
}

pub fn get_buh_home() -> Result<std::path::PathBuf, Error> {
    if let Ok(val) = std::env::var(ENV_BUH_HOME) {
        info!("Использую BUH_HOME из переменной окружения: {val}");
        Ok(std::path::PathBuf::from(val))
    } else {
        let default = dirs::home_dir()
            .map(|h| h.join(DEFAULT_HOME))
            .ok_or(Error::NoConfig)?;
        info!(
            "BUH_HOME не задан, использую директорию по умолчанию: {}",
            default.display()
        );
        Ok(default)
    }
}
