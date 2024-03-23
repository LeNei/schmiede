use anyhow::{Context, Result};
use git2;
use std::fs;
use std::path::Path;

const BASE_REPO_URL: &str = "https://github.com/LeNei/schmiede";

/// Define the starter template in Loco repository
const STARTER_TEMPLATE_FOLDER: &str = "starters";

const TEMP_FOLDER: &str = "./temporary";

pub fn initialize_starter(project_name: &str) -> Result<()> {
    let temp_dir = Path::new(TEMP_FOLDER);
    clone_repo_with_git2(&temp_dir)?;

    let source_folder = temp_dir.join(format!(
        "{}/{}",
        STARTER_TEMPLATE_FOLDER, "axum_sqlx_auth_admin-frontend"
    ));
    fs::rename(source_folder, project_name).context("Failed to get starter")?;

    fs::remove_dir_all(temp_dir).context("Failed to remove repo")?;

    Ok(())
}

fn clone_repo_with_git2(temp_clone_dir: &Path) -> Result<()> {
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.depth(1);
    git2::build::RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(BASE_REPO_URL, temp_clone_dir)?;
    Ok(())
}
