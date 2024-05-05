use crate::{
    add::{AddFeature, FileEditor},
    config::DatabaseType,
};
use anyhow::{Context, Result};
use askama::Template;
use std::{fs, path::Path};
use toml_edit::{value, Array, DocumentMut};

#[derive(Template)]
#[template(path = "./add/database/sqlx.rs.templ", escape = "html")]
pub struct SqlxConfigTemplate {
    pub database: DatabaseType,
}

impl SqlxConfigTemplate {
    pub fn new(database: DatabaseType) -> Self {
        Self { database }
    }

    fn dependencies(&self) -> Vec<(&str, &str, Option<Vec<&str>>)> {
        let db = match self.database {
            DatabaseType::PostgreSQL => "postgres",
        };
        vec![
            (
                "sqlx",
                "0.7.4",
                Some(vec!["runtime-tokio-rustls", "macros", "migrate", db]),
            ),
            ("secrecy", "0.8.0", Some(vec!["serde"])),
            ("serde-aux", "4.1.2", None),
        ]
    }

    fn write_dependencies(&self, path: &Path) -> Result<()> {
        let toml_path = path.join("Cargo.toml");
        let toml_contents =
            fs::read_to_string(&toml_path).with_context(|| "Failed to read Cargo.toml")?;

        let mut manifest = toml_contents
            .parse::<DocumentMut>()
            .with_context(|| "Failed to parse Cargo.toml")?;

        let dependencies = manifest
            .get_mut("dependencies")
            .ok_or(anyhow::anyhow!("Failed to get dependencies"))?;

        for (name, version, features) in self.dependencies() {
            if let Some(features) = features {
                dependencies[name]["version"] = value(version);
                let mut array = Array::default();
                for feature in features {
                    array.push(feature);
                }

                dependencies[name]["features"] = value(array);
            } else {
                dependencies[name] = value(version.to_string());
            }
        }

        let updated_toml = manifest.to_string();
        fs::write(toml_path, updated_toml).with_context(|| "Failed to write Cargo.toml")?;
        Ok(())
    }

    fn write_config(&self, path: &Path) -> Result<()> {
        let rendered = self.render().with_context(|| "Failed to render sqlx.rs")?;
        fs::write(path.join("src/config/database.rs"), rendered)
            .with_context(|| "Failed to write sqlx.rs")?;
        Ok(())
    }

    fn update_config(&self, path: &Path) -> Result<()> {
        let config_path = path.join("src/config/mod.rs");

        let add_use = |lines: &mut Vec<&str>, _: usize| {
            lines.insert(3, "use database::DatabaseSettings;");
            lines.insert(3, "use sqlx::PgPool;");
            lines.insert(0, "mod database;");
        };

        let update_settings = |lines: &mut Vec<&str>, i: usize| {
            lines.insert(i + 1, "    pub database: DatabaseSettings,");
        };

        let update_context = |lines: &mut Vec<&str>, i: usize| {
            lines.insert(i + 1, "    pub db: PgPool,");
        };

        let add_context = |lines: &mut Vec<&str>, has_been_called: Vec<bool>| {
            if has_been_called[has_been_called.len() - 1] {
                return;
            }
            lines.push("#[derive(Clone)]");
            lines.push("pub struct ApiContext {");
            lines.push("    pub db: PgPool,");
            lines.push("}");
        };

        FileEditor::new(&config_path)
            .before_change(add_use)
            .add_change(update_settings, vec!["pub struct Settings {"])
            .add_change(update_context, vec!["pub struct ApiContext {"])
            .after_change(add_context)
            .edit_file()?;

        Ok(())
    }

    fn update_startup(&self, path: &Path) -> Result<()> {
        let startup_path = path.join("./src/startup.rs");

        let update_context = |lines: &mut Vec<&str>, i: usize| {
            lines.insert(i + 1, "    db: settings.database.get_connection_pool(),");
        };

        FileEditor::new(&startup_path)
            .add_change(update_context, vec!["let api_context = ApiContext {"])
            .after_change(|lines, has_been_called| {
                if has_been_called[0] {
                    return;
                }
                lines.insert(0, "use crate::config::ApiContext;");
                let pos = lines
                    .iter()
                    .position(|line| line.contains("pub async fn build"));
                if let Some(pos) = pos {
                    lines.insert(pos + 1, "    let api_context = ApiContext {");
                    lines.insert(
                        pos + 2,
                        "         db: settings.database.get_connection_pool(),",
                    );
                    lines.insert(pos + 3, "    };");
                }

                let pos = lines
                    .iter()
                    .position(|line| line.contains("nest(\"/api\", api_routes())"));
                if let Some(pos) = pos {
                    lines.insert(pos + 1, "       .with_state(api_context.clone())");
                }
            })
            .edit_file()?;

        Ok(())
    }

    fn update_routes(&self, path: &Path) -> Result<()> {
        // TODO: Make recursive for all files in routes directory
        let routes_path = path.join("./src/routes/mod.rs");

        let update_router = |lines: &mut Vec<&str>, i: usize| {
            if let Some(line) = lines.get_mut(i) {
                *line = "pub fn routes() -> Router<ApiContext> {";
                lines.insert(0, "use crate::config::ApiContext;");
            }
        };

        FileEditor::new(&routes_path)
            .add_change(update_router, vec!["pub fn routes() -> Router {"])
            .edit_file()?;

        Ok(())
    }
}

impl AddFeature for SqlxConfigTemplate {
    fn add_feature(&self, path: &Path) -> Result<()> {
        self.write_dependencies(path)?;
        self.write_config(path)?;
        self.update_config(path)?;
        self.update_startup(path)?;
        self.update_routes(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dependencies() {
        let database = DatabaseType::PostgreSQL;
        let template = SqlxConfigTemplate::new(database);
        assert_eq!(
            template.dependencies(),
            vec![
                (
                    "sqlx",
                    "0.7.4",
                    Some(vec![
                        "runtime-tokio-rustls",
                        "macros",
                        "migrate",
                        "offline",
                        "postgres"
                    ])
                ),
                ("secrecy", "0.8.0", Some(vec!["serde"])),
                ("serde-aux", "4.1.2", None),
            ]
        );
    }

    /*
     * Should be done in integration tests
    #[test]
    fn test_write_dependencies() {
        let database = DatabaseType::Postgres;
        let template = SqlxConfigTemplate::new(database);
        template.write_dependencies().unwrap();
    }

    #[test]
    fn test_render_sqlx() {
        let database = DatabaseType::Postgres;
        let template = SqlxConfigTemplate::new(database);
        template.render_sqlx().unwrap();
    }
    */
}
