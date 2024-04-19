use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs;
use std::path::Path;

const BASE_REPO_URL: &str = "https://github.com/LeNei/schmiede";

const STARTER_TEMPLATE_FOLDER: &str = "starters";

const TEMP_FOLDER: &str = "./temporary";

const STARTER_NAMES: [&str; 1] = ["axum"];

#[derive(ValueEnum, Clone, Debug)]
pub enum Starters {
    AxumDieselAuthAdmin,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    #[clap(short, long)]
    pub project_name: Option<String>,

    #[clap(short, long, value_enum)]
    pub template: Option<Starters>,
}

pub fn init_starter(args: InitArgs, term: Term, theme: ColorfulTheme) -> Result<()> {
    let project_name = match args.project_name {
        Some(name) => name,
        None => {
            let name: String = Input::with_theme(&theme)
                .with_prompt("What is the name of your project?")
                .interact_on(&term)
                .unwrap();
            name
        }
    };

    let starter_name = match args.template {
        Some(template) => STARTER_NAMES[template as usize],
        None => {
            let id = Select::with_theme(&theme)
                .with_prompt("Which starter template do you want to use?")
                .items(&STARTER_NAMES)
                .interact_on(&term)
                .unwrap();
            STARTER_NAMES[id]
        }
    };

    let database = Select::with_theme(&theme)
        .with_prompt("Which database do you want to use?")
        .items(&["Postgres", "Sqlite"])
        .interact_on(&term)
        .unwrap();

    create_starter(&project_name, starter_name)
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
