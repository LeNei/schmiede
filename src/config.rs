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

#[derive(Deserialize, Default, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiFramework {
    #[default]
    Axum,
    //Actix,
}

impl ApiFramework {
    pub fn values() -> [&'static str; 1] {
        ["axum"]
    }
}

impl FromStr for ApiFramework {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "axum" => Ok(ApiFramework::Axum),
            //"actix" => Ok(ApiFramework::Actix),
            _ => anyhow::bail!("Failed to get api framework from str"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Database {
    Postgres,
    //Sqlite,
}

impl Database {
    pub fn values() -> [&'static str; 1] {
        ["postgres"]
    }
}

impl FromStr for Database {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "postgres" => Ok(Database::Postgres),
            //"sqlite" => Ok(Database::Sqlite),
            _ => anyhow::bail!("Failed to get database from str"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseDriver {
    Sqlx,
    Diesel,
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
            "sqlx" => Ok(DatabaseDriver::Sqlx),
            "diesel" => Ok(DatabaseDriver::Diesel),
            _ => anyhow::bail!("Failed to get database driver from str"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_framework_from_values() {
        let values = ApiFramework::values();
        let invalid = "invalid";

        for value in values.iter() {
            assert!(ApiFramework::from_str(value).is_ok());
        }
        assert!(ApiFramework::from_str(invalid).is_err());
    }

    #[test]
    fn test_database_from_values() {
        let values = Database::values();
        let invalid = "invalid";

        for value in values.iter() {
            assert!(Database::from_str(value).is_ok());
        }
        assert!(Database::from_str(invalid).is_err());
    }

    #[test]
    fn test_database_driver_from_values() {
        let values = DatabaseDriver::values();
        let invalid = "invalid";

        for value in values.iter() {
            assert!(DatabaseDriver::from_str(value).is_ok());
        }
        assert!(DatabaseDriver::from_str(invalid).is_err());
    }

    #[test]
    fn test_config_from_file() {
        let config = Config::from_file();
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_write_to_file() {
        let mut config = Config::new().unwrap();
        //config.add_api_framework(ApiFramework::Actix);
        config.add_database(Database::Postgres);
        config.add_database_driver(DatabaseDriver::Diesel);
        let write = config.write_to_file();
        assert!(write.is_ok());
    }

    #[test]
    fn test_config_add_api_framework() {
        let mut config = Config::new().unwrap();
        config.add_api_framework(ApiFramework::Axum);
        assert_eq!(config.api_framework, ApiFramework::Axum);
    }

    #[test]
    fn test_config_add_database() {
        let mut config = Config::new().unwrap();
        config.add_database(Database::Postgres);
        assert_eq!(config.database, Some(Database::Postgres));
    }

    #[test]
    fn test_config_add_database_driver() {
        let mut config = Config::new().unwrap();
        config.add_database_driver(DatabaseDriver::Diesel);
        assert_eq!(config.database_driver, Some(DatabaseDriver::Diesel));
    }
}
