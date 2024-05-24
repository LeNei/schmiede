use crate::{
    add::{add_dependencies, write_config, AddFeature, Dependency, FileEditor},
    config::DatabaseType,
};
use anyhow::Result;
use askama::Template;
use std::path::Path;

use super::update_routes;

#[derive(Template)]
#[template(path = "./add/database/diesel.rs.templ", escape = "html")]
pub struct DieselConfigTemplate {
    pub database: DatabaseType,
}

impl DieselConfigTemplate {
    pub fn new(database: DatabaseType) -> Self {
        Self { database }
    }

    fn dependencies(&self) -> Vec<Dependency> {
        let db = match self.database {
            DatabaseType::PostgreSQL => "postgres",
        };

        vec![
            ("diesel", "2.1.0", Some(vec![db])),
            ("diesel-async", "0.4.1", Some(vec![db, "deadpool"])),
            ("secrecy", "0.8.0", Some(vec!["serde"])),
            ("serde-aux", "4.1.2", None),
        ]
    }

    fn update_config(&self, path: &Path) -> Result<()> {
        let add_use = |lines: &mut Vec<&str>| {
            lines.insert(3, "use database::{DatabaseSettings, PgPool};");
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

        FileEditor::new(&path.join("src/config/mod.rs"))
            .before_change(add_use)
            .add_change(update_settings, vec!["pub struct Settings {"])
            .add_change(update_context, vec!["pub struct ApiContext {"])
            .after_change(add_context)
            .edit_file()?;

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
            .create_file("postgresql://postgres:postgres@localhost:5432/postgres")?;

        Ok(())
    }

    fn update_startup(&self, path: &Path) -> Result<()> {
        let startup_path = path.join("./src/startup.rs");

        let update_context = |lines: &mut Vec<&str>, i: usize| {
            lines.insert(i + 1, "    db: settings.database.get_connection_pool().context(\"Failed to connect to database\")?,");
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
                        "         db: settings.database.get_connection_pool().context(\"Failed to connect to database\")?,",
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
}

impl AddFeature for DieselConfigTemplate {
    fn add_feature(&self, path: &Path) -> Result<()> {
        add_dependencies(path, self.dependencies())?;
        write_config(&path.join("src/config/database.rs"), self)?;
        self.update_config(path)?;
        self.update_startup(path)?;
        update_routes(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dependencies() {
        let database = DatabaseType::PostgreSQL;
        let template = DieselConfigTemplate::new(database);
        let db = "postgres";
        assert_eq!(
            template.dependencies(),
            vec![
                ("diesel", "2.1.0", Some(vec![db])),
                ("diesel_async", "0.4.1", Some(vec![db, "deadpool"])),
                ("secrecy", "0.8.0", Some(vec!["serde"])),
                ("serde_aux", "4.1.2", None),
            ]
        );
    }

    /*
     * Should be done in integration tests
    #[test]
    fn test_write_dependencies() {
        let database = DatabaseType::Postgres;
        let template = DieselConfigTemplate::new(database);
        template.write_dependencies().unwrap();
    }

    #[test]
    fn test_render_sqlx() {
        let database = DatabaseType::Postgres;
        let template = DieselConfigTemplate::new(database);
        template.render_sqlx().unwrap();
    }
    */
}
