use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub database: Database,
    pub endpoint: Endpoint,
    pub jwt: Jwt,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Jwt {
    pub secret: String,
}

impl Settings {
    pub fn new(name: &str) -> Result<Settings, ConfigError> {
        Config::builder()
            .add_source(File::with_name(name))
            .build()?
            .try_deserialize()
    }
}
