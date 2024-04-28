use super::UpdateDatabaseFiles;
use crate::{add::AddFeature, config::Database};
use anyhow::{Context, Result};
use askama::Template;
use std::fs;
use toml_edit::{value, Array, DocumentMut};

#[derive(Template)]
#[template(path = "./add/database/sqlx.rs.templ", escape = "html")]
pub struct SqlxConfigTemplate {
    pub database: Database,
}

impl SqlxConfigTemplate {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    fn dependencies(&self) -> Vec<(&str, &str, Option<Vec<&str>>)> {
        let db = match self.database {
            Database::Postgres => "postgres",
            Database::None => panic!("Database not supported"),
        };
        // TODO: Add support for other databases
        vec![
            (
                "sqlx",
                "0.7.4",
                Some(vec![
                    "runtime-tokio-rustls",
                    "macros",
                    "migrate",
                    "offline",
                    db,
                ]),
            ),
            ("secrecy", "0.8.0", Some(vec!["serde"])),
            ("serde-aux", "4.1.2", None),
        ]
    }

    fn write_dependencies(&self) -> Result<()> {
        let toml_contents =
            fs::read_to_string("Cargo.toml").with_context(|| "Failed to read Cargo.toml")?;

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
        fs::write("Cargo.toml", updated_toml).with_context(|| "Failed to write Cargo.toml")?;
        Ok(())
    }

    fn write_config(&self) -> Result<()> {
        let rendered = self.render().with_context(|| "Failed to render sqlx.rs")?;
        fs::write("src/config/database.rs", rendered).with_context(|| "Failed to write sqlx.rs")?;
        Ok(())
    }
}

impl UpdateDatabaseFiles for SqlxConfigTemplate {
    fn update_config() -> Result<bool> {
        let config = fs::read_to_string("./src/config/mod.rs")
            .with_context(|| "Failed to read config file")?;

        let mut lines = config.lines().collect::<Vec<_>>();
        lines.insert(1, "pub mod database;");

        let mut found_use = false;
        let mut found_struct = false;
        let mut found_context = false;

        for (i, line) in lines.clone().iter().enumerate() {
            if found_use && found_struct && found_context {
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

            if !found_context && line.contains("pub struct ApiContext {") {
                lines.insert(i + 1, "    pub db: PgPool,");
                found_context = true;
                continue;
            }
        }

        if !found_context {
            lines.push("pub struct ApiContext {");
            lines.push("    pub db: PgPool,");
            lines.push("}");
        }

        let updated_config = lines.join("\n");
        fs::write("./src/config/mod.rs", updated_config)
            .with_context(|| "Failed to write updated config file")?;

        Ok(found_context)
    }

    fn update_startup(has_context: bool) -> Result<()> {
        let startup = fs::read_to_string("./src/startup.rs")
            .with_context(|| "Failed to read startup file")?;

        let mut lines = startup.lines().collect::<Vec<_>>();

        let mut found_context = false;
        let mut found_tracing = false;

        if !has_context {
            lines.insert(0, "use crate::config::ApiContext;");
        }

        for (i, line) in lines.clone().iter().enumerate() {
            if found_context && found_tracing {
                break;
            }

            if !has_context {
                if !found_context && line.contains("let api_context = ApiContext {") {
                    lines.insert(i + 1, "    db: settings.database.get_connection_pool(),");
                    found_context = true;
                    continue;
                }
            } else {
                if !found_context && line.contains("pub async fn build") {
                    lines.insert(i, "let api_context = ApiContext {");
                    lines.insert(i + 1, "    db: settings.database.get_connection_pool().context(\"Failed to connect to database\")?,");
                    lines.insert(i + 2, "};");

                    found_context = true;
                    continue;
                }
            }

            if !found_tracing && line.contains("TraceLayer") {
                let line = lines.get_mut(i).unwrap().replace(';', "");
                lines.insert(i, "   .with_state(api_context.clone());");
                found_tracing = true;
                continue;
            }
        }
        Ok(())
    }
}

impl AddFeature for SqlxConfigTemplate {
    fn add_feature(&self) -> Result<()> {
        self.write_dependencies()?;
        self.write_config()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dependencies() {
        let database = Database::Postgres;
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
        let database = Database::Postgres;
        let template = SqlxConfigTemplate::new(database);
        template.write_dependencies().unwrap();
    }

    #[test]
    fn test_render_sqlx() {
        let database = Database::Postgres;
        let template = SqlxConfigTemplate::new(database);
        template.render_sqlx().unwrap();
    }
    */
}
