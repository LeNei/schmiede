use anyhow::{Context, Result};
use askama::Template;
use std::fs::File;
use std::io::Write;

use crate::data_types::IDType;

pub trait CreateTemplate {
    fn create_template(&self, file_path: String) -> Result<()>;
}

impl<T: Template> CreateTemplate for T {
    fn create_template(&self, file_path: String) -> Result<()> {
        let mut file = File::create(file_path).context("Failed to create file")?;

        file.write_all(
            self.render()
                .context("Failed to render template")?
                .as_bytes(),
        )
        .context("Failed to write sql down template to file")
    }
}

#[derive(Template)]
#[template(path = "db.up.sql.templ", escape = "html")]
pub struct DbUpTemplate<'a> {
    pub name: &'a str,
    pub rows: Vec<String>,
    pub id: IDType,
}

#[derive(Template)]
#[template(path = "db.down.sql.templ", escape = "html")]
pub struct DbDownTemplate<'a> {
    pub name: &'a str,
}

#[derive(Template)]
#[template(path = "model.rs.templ", escape = "html")]
pub struct ModelTemplate<'a> {
    pub id: IDType,
    pub name: &'a str,
    pub struct_name: &'a str,
    pub rows: Vec<String>,
}

#[derive(Template)]
#[template(path = "page.rs.templ", escape = "html")]
pub struct PageTemplate<'a> {
    pub function_name: &'a str,
    pub model_name: &'a str,
    pub route: &'a str,
}
/*
#[derive(Template)]
#[template(path = "api.rs", escape = "none")]
pub struct ApiTemplate<'a> {
    pub struct_name: &'a str,
    pub function_name: &'a str,
    pub rows: Vec<String>,
    pub edit_rows: Vec<String>,
    pub id: IDType,
}
*/
