use crate::{
    add::{add_dependencies, write_config, AddFeature, Dependency, FileEditor},
    config::DatabaseType,
};
use anyhow::Result;
use askama::Template;
use std::{fs, path::Path};

use super::{update_config_files, update_routes};

#[derive(Template)]
#[template(path = "./add/database/sqlx.rs.templ", escape = "html")]
pub struct SqlxConfigTemplate {
    pub database: DatabaseType,
}

impl SqlxConfigTemplate {
    pub fn new(database: DatabaseType) -> Self {
        Self { database }
    }

    fn dependencies(&self) -> Vec<Dependency> {
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

    fn update_config(&self, path: &Path) -> Result<()> {
        let add_use = |lines: &mut Vec<&str>| {
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

        FileEditor::new(&path.join("src/config/mod.rs"))
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

        update_config_files(path)?;

        Ok(())
    }

    fn add_updated_at(&self, path: &Path) -> Result<()> {
        fs::create_dir(path.join("migrations"))?;
        FileEditor::new(&path.join("migrations/20210101000000_initial_setup.up.sql")).create_file(
            r#"
-- add function for updated_at

CREATE OR REPLACE FUNCTION manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE sel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION sel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

"#,
        )?;
        FileEditor::new(&path.join("migrations/20210101000000_initial_setup.down.sql"))
            .create_file(
                r#"
-- remove function for updated_at
DROP FUNCTION IF EXISTS manage_updated_at(_tbl regclass);

DROP FUNCTION IF EXISTS set_updated_at();

"#,
            )?;
        Ok(())
    }
}

impl AddFeature for SqlxConfigTemplate {
    fn add_feature(&self, path: &Path) -> Result<()> {
        add_dependencies(path, self.dependencies())?;
        write_config(&path.join("src/config/database.rs"), self)?;
        self.update_config(path)?;
        self.update_startup(path)?;
        self.add_updated_at(path)?;
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
