mod diesel;
mod sqlx;

use crate::add::AddFeature;
use anyhow::{Context, Result};
use std::fs;

trait UpdateDatabaseFiles {
    fn update_config() -> Result<bool>;
    fn update_startup(has_context: bool) -> Result<()>;
}

fn update_config() -> Result<()> {
    let config =
        fs::read_to_string("./src/config/mod.rs").with_context(|| "Failed to read config file")?;

    let mut lines = config.lines().collect::<Vec<_>>();
    lines.insert(1, "pub mod database;");

    let mut found_use = false;
    let mut found_struct = false;

    for (i, line) in lines.clone().iter().enumerate() {
        if found_use && found_struct {
            break;
        }
        if !found_use && line.contains("use") {
            lines.insert(i, "use database::DatabaseSettings;");
            lines.insert(i, "use sqlx::PgPool;");
            found_use = true;
            continue;
        }
        if !found_struct && line.contains("pub struct Settings {") {
            lines.insert(i + 1, "    pub database: DatabaseSettings,");
            found_struct = true;
            continue;
        }
    }

    let updated_config = lines.join("\n");
    fs::write("./src/config/mod.rs", updated_config)
        .with_context(|| "Failed to write updated config file")?;

    Ok(())
}

fn update_startup() -> Result<()> {
    let startup =
        fs::read_to_string("./src/startup.rs").with_context(|| "Failed to read startup file")?;

    let mut lines = startup.lines().collect::<Vec<_>>();
    Ok(())
}

fn add_database<T: AddFeature>(feature: T) -> Result<()> {
    feature.add_feature()?;
    update_config()?;

    Ok(())
}
