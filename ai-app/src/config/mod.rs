mod error;

use std::path::Path;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub use error::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: Server,
    pub database: Database,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Database {
    #[serde(rename = "sqlite")]
    Sqlite { name: String },
}

impl Database {
    pub fn connection_string(&self) -> &str {
        match self {
            Database::Sqlite { name } => name,
        }
    }
}

/// Read and deserialize a JSON file at `path`.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::FileNotFound {
                path: path.to_path_buf(),
            }
        } else {
            Error::IoError(e)
        }
    })?;

    serde_json::from_str(&content).map_err(|e| Error::ParseError {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Serialize and write a JSON file at `path`.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
    let content =
        serde_json::to_string_pretty(value).map_err(|e| Error::ParseError {
            path: path.to_path_buf(),
            source: e,
        })?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::{Config, Database, Error, Server, read_json, write_json};

    #[test]
    fn test_sqlite_config_roundtrip() {
        let config = Config {
            server: Server {
                host: "127.0.0.1".to_string(),
                port: 31415,
            },
            database: Database::Sqlite {
                name: "test.db".to_string(),
            },
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.server.host, "127.0.0.1");
        assert_eq!(parsed.server.port, 31415);
        assert!(
            matches!(parsed.database, Database::Sqlite { ref name } if name == "test.db")
        );
    }

    #[test]
    fn test_config_json_format() {
        let json = r#"{
            "server": { "host": "0.0.0.0", "port": 8080 },
            "database": { "type": "sqlite", "name": "my.db" },
            "log_file": "/var/log/app.log"
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.server.port, 8080);
        assert!(
            matches!(config.database, Database::Sqlite { ref name } if name == "my.db")
        );
    }

    #[test]
    fn test_read_json_file_not_found() {
        let result = read_json::<Config>(Path::new("/nonexistent/config.json"));
        assert!(matches!(result, Err(Error::FileNotFound { .. })));
    }

    #[test]
    fn test_write_and_read_json() {
        let dir = std::env::temp_dir().join("ai_app_config_test");
        let path = dir.join("config.json");

        let config = Config {
            server: Server {
                host: "localhost".to_string(),
                port: 3000,
            },
            database: Database::Sqlite {
                name: "test.db".to_string(),
            },
        };

        write_json(&path, &config).unwrap();
        let loaded: Config = read_json(&path).unwrap();

        assert_eq!(loaded.server.host, "localhost");
        assert_eq!(loaded.server.port, 3000);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
