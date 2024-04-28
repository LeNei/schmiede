use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::path::Path;
use std::{fs, str::FromStr};

use crate::config::{ApiFramework, Database};

const BASE_REPO_URL: &str = "https://github.com/LeNei/schmiede";

const STARTER_TEMPLATE_FOLDER: &str = "starters";

const TEMP_FOLDER: &str = "./temporary";

const STARTER_NAMES: [&str; 1] = ["axum"];

#[derive(ValueEnum, Clone, Debug)]
pub enum Starters {
    Axum,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    #[clap(short, long)]
    pub project_name: Option<String>,

    #[clap(short, long, value_enum)]
    pub template: Option<ApiFramework>,

    #[clap(short, long, value_enum)]
    pub database: Option<Database>,
}

pub fn init_starter(args: InitArgs, term: Term, theme: ColorfulTheme) -> Result<()> {
    let project_name = match args.project_name {
        Some(name) => name,
        None => Input::with_theme(&theme)
            .with_prompt("What is the name of your project?")
            .interact_on(&term)
            .context("Failed to get project name")?,
    };

    let starter_name = match args.template {
        Some(template) => template,
        None => {
            let index = Select::with_theme(&theme)
                .with_prompt("Which starter template do you want to use?")
                .items(&ApiFramework::VALUES)
                .interact_on(&term)
                .context("Failed to get starter")?;
            ApiFramework::from_str(ApiFramework::VALUES[index])
                .context("Failed to parse starter")?
        }
    };

    let database = match args.database {
        Some(database) => database,
        None => {
            let index = Select::with_theme(&theme)
                .with_prompt("Which database do you want to use?")
                .items(&Database::VALUES)
                .interact_on(&term)
                .context("Failed to get database")?;
            Database::from_str(Database::VALUES[index]).context("Failed to parse database")?
        }
    };

    //create_starter(&project_name, starter_name)
    Ok(())
}

pub fn create_starter(project_name: &str, template_name: &str) -> Result<()> {
    let temp_dir = Path::new(TEMP_FOLDER);

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.depth(1);
    git2::build::RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(BASE_REPO_URL, temp_dir)?;

    let source_folder = temp_dir.join(format!("{}/{}", STARTER_TEMPLATE_FOLDER, template_name));
    fs::rename(source_folder, project_name).context("Failed to get starter")?;

    fs::remove_dir_all(temp_dir).context("Failed to remove repo")?;

    Ok(())
}
