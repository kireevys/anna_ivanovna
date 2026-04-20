use std::path::{Path, PathBuf};

use ai_app::config::{Config, Database, Error, Server, read_json, write_json};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct TauriDefaults {
    pub buh_home: String,
    pub database: Database,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TauriSettings {
    pub buh_home: PathBuf,
}

pub struct TauriConfigProvider {
    app_config_dir: PathBuf,
}

impl TauriConfigProvider {
    pub fn new(app_config_dir: PathBuf) -> Self {
        Self { app_config_dir }
    }

    fn settings_path(&self) -> PathBuf {
        self.app_config_dir.join("settings.json")
    }

    pub fn load_defaults() -> Result<TauriDefaults, Error> {
        let json = include_str!("../defaults/config.json");
        serde_json::from_str(json).map_err(|e| Error::ParseError {
            path: PathBuf::from("defaults/config.json"),
            source: e,
        })
    }

    pub fn has_settings(&self) -> bool {
        self.settings_path().exists()
    }

    pub fn default_buh_home() -> Result<PathBuf, Error> {
        let defaults = Self::load_defaults()?;
        Self::resolve_buh_home_path(&defaults.buh_home)
    }

    pub fn resolve_buh_home_path(raw: &str) -> Result<PathBuf, Error> {
        let path = Path::new(raw);
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            let home = dirs::home_dir().ok_or(Error::HomeDirNotFound)?;
            Ok(home.join(raw))
        }
    }

    pub fn resolve_buh_home(&self) -> Result<PathBuf, Error> {
        let settings: TauriSettings = read_json(&self.settings_path())?;
        Ok(settings.buh_home)
    }

    pub fn load(&self) -> Result<Config, Error> {
        let buh_home = self.resolve_buh_home()?;
        read_json(&buh_home.join("config.json"))
    }

    pub fn save(&self, config: &Config) -> Result<(), Error> {
        let buh_home = self.resolve_buh_home()?;
        write_json(&buh_home.join("config.json"), config)
    }

    pub fn save_initial(
        &self,
        buh_home: &Path,
        database: Database,
    ) -> Result<(), Error> {
        let database = match database {
            Database::Sqlite { name } => {
                let abs = buh_home.join(&name).to_string_lossy().to_string();
                Database::Sqlite { name: abs }
            }
        };
        let config = Config {
            server: Server {
                host: "127.0.0.1".to_string(),
                port: 31415,
            },
            database,
        };
        std::fs::create_dir_all(buh_home)?;
        let config_path = buh_home.join("config.json");
        write_json(&config_path, &config)?;
        info!("Saved config to {}", config_path.display());

        let settings = TauriSettings {
            buh_home: buh_home.to_path_buf(),
        };
        std::fs::create_dir_all(&self.app_config_dir)?;
        write_json(&self.settings_path(), &settings)?;
        info!("Saved settings to {}", self.settings_path().display());

        Ok(())
    }
}
