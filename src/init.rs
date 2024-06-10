use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    add::{add_addon, Features},
    config::{ApiFramework, Config, ConfigBuilder, Database, DatabaseDriver, DatabaseType},
    generate::FromTerm,
};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::ProgressBar;

#[derive(ValueEnum, Clone, Debug)]
pub enum Starters {
    Axum,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    #[clap(short, long)]
    pub name: Option<String>,

    #[clap(short, long, value_enum)]
    pub api_framework: Option<ApiFramework>,

    #[clap(short = 't', long, value_enum)]
    pub database_type: Option<DatabaseType>,

    #[clap(short = 'd', long, value_enum)]
    pub database_driver: Option<DatabaseDriver>,
}

pub fn init_starter(args: InitArgs, term: Term, theme: ColorfulTheme) -> Result<()> {
    let project_name = match args.name {
        Some(name) => name,
        None => Input::with_theme(&theme)
            .with_prompt("What is the name of your project?")
            .interact_on(&term)
            .context("Failed to get project name")?,
    };

    if Path::new(&project_name).exists() {
        anyhow::bail!("Project folder already exists");
    }

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
        .database(database.clone())
        .build();

    let pb_starter = ProgressBar::new_spinner();
    pb_starter.set_message("Creating project...");
    pb_starter.enable_steady_tick(Duration::from_millis(120));
    let path = clone_starter(&config, &project_name)?;
    pb_starter.set_message("Creating project ✓");
    pb_starter.finish();

    let pb_addons = ProgressBar::new_spinner();

    if database.is_some() {
        pb_addons.set_message("Preparing addons...");
        pb_addons.enable_steady_tick(Duration::from_millis(120));
        add_addon(Features::Database(database.unwrap()), false)?;
        pb_addons.set_message("Preparing addons ✓");
        pb_addons.finish();
    }

    config.create_config_toml(&path)?;
    Ok(())
}

fn clone_starter(config: &Config, project_name: &str) -> Result<PathBuf> {
    let temp_dir = Path::new("./temporary");
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.depth(1);
    git2::build::RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone("https://github.com/LeNei/schmiede", temp_dir)
        .context("Failed to clone repo")?;

    let source_folder = temp_dir.join(format!("{}/{}", "starters", config.api_framework));

    fs::rename(source_folder, project_name).context("Failed to get starter")?;

    fs::remove_dir_all(temp_dir).context("Failed to remove temporary folder")?;
    Ok(Path::new(project_name).to_path_buf())
}
