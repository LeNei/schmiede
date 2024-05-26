pub mod diesel;
pub mod sqlx;

use super::FileEditor;
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

fn update_config_files(path: &Path) -> Result<()> {
    let add_config = |lines: &mut Vec<&str>, has_been_called: Vec<bool>| {
        if has_been_called[0] {
            return;
        }
        lines.push("database:");
        lines.push("  username: postgres");
        lines.push("  password: postgres");
        lines.push("  port: 5432");
        lines.push("  host: localhost");
        lines.push("  database_name: postgres");
        lines.push("  require_ssl: false");
    };

    FileEditor::new(&path.join("configuration/base.yaml"))
        .add_change(|_, _| {}, vec!["database:"])
        .after_change(add_config)
        .edit_file()?;

    FileEditor::new(&path.join(".env.local"))
        .create_file("DATABASE_URL=postgresql://postgres:postgres@localhost:5432/postgres")?;

    Ok(())
}

fn update_routes(path: &Path) -> Result<()> {
    let file_pattern = ".rs";

    for entry in WalkDir::new(path.join("src/routes/"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();

        if !file_path.is_file() || !file_path.to_string_lossy().ends_with(file_pattern) {
            continue; // Skip non-file entries or files that don't match the pattern
        }

        let update_router = |lines: &mut Vec<&str>, i: usize| {
            if let Some(line) = lines.get_mut(i) {
                *line = "pub fn routes() -> Router<ApiContext> {";
                lines.insert(0, "use crate::config::ApiContext;");
            }
        };

        FileEditor::new(file_path)
            .add_change(update_router, vec!["pub fn routes() -> Router {"])
            .edit_file()?;
    }
    Ok(())
}
