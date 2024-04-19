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

    #[test]
    fn test_write_dependencies() {
        let database = Database::Postgres;
        let template = SqlxConfigTemplate::new(database);
        template.write_dependencies().unwrap();
    }

    /*
    #[test]
    fn test_render_sqlx() {
        let database = Database::Postgres;
        let template = SqlxConfigTemplate::new(database);
        template.render_sqlx().unwrap();
    }
    */
}
