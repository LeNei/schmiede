use crate::{
    add::{add_dependencies, write_config, AddFeature, Dependency},
    config::DatabaseType,
};
use anyhow::Result;
use askama::Template;
use std::path::Path;

#[derive(Template)]
#[template(path = "./add/database/sqlx.rs.templ", escape = "html")]
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
            ("diesel_async", "0.4.1", Some(vec![db, "deadpool"])),
            ("secrecy", "0.8.0", Some(vec!["serde"])),
            ("serde_aux", "4.1.2", None),
        ]
    }
}

impl AddFeature for DieselConfigTemplate {
    fn add_feature(&self, path: &Path) -> Result<()> {
        add_dependencies(path, self.dependencies())?;
        write_config(&path.join("src/config/database.rs"), self)?;
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
