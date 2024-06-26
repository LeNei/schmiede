use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::{default::Default, fs, str::FromStr};

use crate::generate::FromTerm;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub api_framework: ApiFramework,
    pub database: Option<Database>,
}

impl Config {
    pub fn from_file() -> Result<Self> {
        let config = fs::read_to_string("schmiede.toml")?;
        let config: Config = toml::from_str(&config)?;
        Ok(config)
    }

    pub fn update_config(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("schmiede.toml")?;

        let config = toml::to_string_pretty(self)?;
        file.write_all(&config.into_bytes())?;
        Ok(())
    }

    pub fn create_config_toml(&self, project_path: &Path) -> Result<()> {
        let config = toml::to_string_pretty(self)?;
        fs::write(project_path.join("schmiede.toml"), config).context("Failed to write config")?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ConfigBuilder {
    #[serde(default)]
    api_framework: ApiFramework,
    database: Option<Database>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn api_framework(&mut self, api_framework: ApiFramework) -> &mut Self {
        self.api_framework = api_framework;
        self
    }

    pub fn database(&mut self, database: Option<Database>) -> &mut Self {
        self.database = database;
        self
    }

    pub fn build(&self) -> Config {
        Config {
            api_framework: self.api_framework.clone(), // Assuming api_framework is now required
            database: self.database.clone(),
        }
    }
}

#[derive(Deserialize, Default, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ApiFramework {
    #[default]
    Axum,
    //Actix,
}

impl Display for ApiFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Axum => write!(f, "axum"),
        }
    }
}

impl FromTerm<ApiFramework> for ApiFramework {
    fn from_term(
        term: &console::Term,
        theme: &dialoguer::theme::ColorfulTheme,
    ) -> Result<ApiFramework> {
        let options = ["Axum"];
        let index = dialoguer::Select::with_theme(theme)
            .with_prompt("Which api framework do you want to use?")
            .items(&options)
            .interact_on(term)?;

        ApiFramework::from_str(options[index])
    }
}

impl FromStr for ApiFramework {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input.to_lowercase().as_str() {
            "axum" => Ok(ApiFramework::Axum),
            //"actix" => Ok(ApiFramework::Actix),
            _ => anyhow::bail!("Failed to get api framework from str"),
        }
    }
}

#[derive(Parser, Deserialize, Serialize, Debug, Clone)]
pub struct Database {
    #[clap(short = 't', long, value_enum)]
    pub database_type: DatabaseType,
    #[clap(short = 'd', long, value_enum)]
    pub database_driver: DatabaseDriver,
}

impl Database {
    pub fn new(database_type: DatabaseType, database_driver: DatabaseDriver) -> Self {
        Self {
            database_type,
            database_driver,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    PostgreSQL,
    //MySQL,
}

impl FromTerm<Option<DatabaseType>> for DatabaseType {
    fn from_term(
        term: &console::Term,
        theme: &dialoguer::theme::ColorfulTheme,
    ) -> Result<Option<DatabaseType>> {
        let options = ["PostgreSQL", "None"];

        let index = dialoguer::Select::with_theme(theme)
            .with_prompt("Which database do you want to use?")
            .items(&options)
            .interact_on(term)?;

        if let Ok(database) = DatabaseType::from_str(options[index]) {
            Ok(Some(database))
        } else {
            Ok(None)
        }
    }
}

impl FromStr for DatabaseType {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input.to_lowercase().as_str() {
            "postgresql" => Ok(DatabaseType::PostgreSQL),
            //"sqlite" => Ok(Database::Sqlite),
            _ => anyhow::bail!("Failed to get database from str"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseDriver {
    Sqlx,
    Diesel,
}

impl FromTerm<DatabaseDriver> for DatabaseDriver {
    fn from_term(
        term: &console::Term,
        theme: &dialoguer::theme::ColorfulTheme,
    ) -> Result<DatabaseDriver> {
        let options = ["Sqlx", "Diesel"];
        let index = dialoguer::Select::with_theme(theme)
            .with_prompt("Which database driver do you want to use?")
            .items(&options)
            .interact_on(term)?;

        DatabaseDriver::from_str(options[index])
    }
}

impl FromStr for DatabaseDriver {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input.to_lowercase().as_str() {
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
        let invalid = "invalid";
        let options = ["axum"];

        for option in options.iter() {
            assert!(ApiFramework::from_str(option).is_ok());
        }

        assert!(ApiFramework::from_str(invalid).is_err());
    }

    #[test]
    fn test_database_driver_from_values() {
        let invalid = "invalid";
        let options = ["Sqlx", "Diesel"];

        for value in options.iter() {
            assert!(DatabaseDriver::from_str(value).is_ok());
        }

        assert!(DatabaseDriver::from_str(invalid).is_err());
    }

    /*
     * Should be tested in integration tests
    #[test]
    fn test_config_from_file() {
        let config = Config::from_file();
        assert!(config.is_ok());
    }
    */

    /*
     * Should be tested in integration tests
    #[test]
    fn test_config_write_to_file() {
        let mut config = Config::new().unwrap();
        //config.add_api_framework(ApiFramework::Actix);
        config.add_database(Database::Postgres);
        config.add_database_driver(DatabaseDriver::Diesel);
        let write = config.write_to_file();
        assert!(write.is_ok());
    }
    */
}
