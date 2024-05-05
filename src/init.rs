use crate::{
    config::{ApiFramework, ConfigBuilder, Database, DatabaseDriver, DatabaseType},
    generate::FromTerm,
};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input};

#[derive(ValueEnum, Clone, Debug)]
pub enum Starters {
    Axum,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    #[clap(short, long)]
    pub project_name: Option<String>,

    #[clap(short, long, value_enum)]
    pub api_framework: Option<ApiFramework>,

    #[clap(short = 't', long, value_enum)]
    pub database_type: Option<DatabaseType>,

    #[clap(short = 'd', long, value_enum)]
    pub database_driver: Option<DatabaseDriver>,
}

pub fn init_starter(args: InitArgs, term: Term, theme: ColorfulTheme) -> Result<()> {
    let project_name = match args.project_name {
        Some(name) => name,
        None => Input::with_theme(&theme)
            .with_prompt("What is the name of your project?")
            .interact_on(&term)
            .context("Failed to get project name")?,
    };

    let api_framework = match args.api_framework {
        Some(a) => a,
        None => ApiFramework::from_term(&term, &theme).context("Failed to get api framework")?,
    };

    let database: Option<Database> = match args.database_type {
        Some(database) => {
            let driver = match args.database_driver {
                Some(d) => d,
                None => DatabaseDriver::from_term(&term, &theme)
                    .context("Failed to get database driver")?,
            };
            Some(Database::new(database, driver))
        }
        None => {
            let database =
                DatabaseType::from_term(&term, &theme).context("Failed to get database")?;
            if database.is_none() {
                None
            } else {
                let driver = DatabaseDriver::from_term(&term, &theme)
                    .context("Failed to get database driver")?;
                Some(Database::new(database.unwrap(), driver))
            }
        }
    };

    let config = ConfigBuilder::new()
        .api_framework(api_framework)
        .database(database)
        .build();

    let path = config.init_from_starter(&project_name)?;
    config.init_addons(&path)?;
    Ok(())
}
