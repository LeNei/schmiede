use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{default::Default, fs, str::FromStr};

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub api_framework: ApiFramework,
    pub database: Option<Database>,
    pub database_driver: Option<DatabaseDriver>,
}

impl Config {
    fn from_file() -> Result<Self> {
        let config = fs::read_to_string("schmiede.toml")?;
        let config: Config = toml::from_str(&config)?;
        Ok(config)
    }

    pub fn new() -> Result<Self> {
        if let Ok(config) = Self::from_file() {
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn add_api_framework(&mut self, api_framework: ApiFramework) {
        self.api_framework = api_framework;
    }

    pub fn add_database(&mut self, database: Database) {
        self.database = Some(database);
    }

    pub fn add_database_driver(&mut self, database_driver: DatabaseDriver) {
        self.database_driver = Some(database_driver);
    }

    pub fn write_to_file(&self) -> Result<()> {
        let config = toml::to_string(self)?;
        fs::write("schmiede.toml", config)?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ApiFramework {
    AXUM,
    ACTIX,
}

impl Default for ApiFramework {
    fn default() -> Self {
        ApiFramework::AXUM
    }
}

impl ApiFramework {
    pub fn values() -> [&'static str; 2] {
        ["axum", "actix"]
    }
}

impl FromStr for ApiFramework {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "axum" => Ok(ApiFramework::AXUM),
            "actix" => Ok(ApiFramework::ACTIX),
            _ => anyhow::bail!("Failed to get api framework from str"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Database {
    POSTGRES,
    SQLITE,
}

impl Database {
    pub fn values() -> [&'static str; 2] {
        ["postgres", "sqlite"]
    }
}

impl FromStr for Database {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "postgres" => Ok(Database::POSTGRES),
            "sqlite" => Ok(Database::SQLITE),
            _ => anyhow::bail!("Failed to get database from str"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseDriver {
    SQLX,
    DIESEL,
}

impl DatabaseDriver {
    pub fn values() -> [&'static str; 2] {
        ["sqlx", "diesel"]
    }
}

impl FromStr for DatabaseDriver {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "sqlx" => Ok(DatabaseDriver::SQLX),
            "diesel" => Ok(DatabaseDriver::DIESEL),
            _ => anyhow::bail!("Failed to get database driver from str"),
        }
    }
}
