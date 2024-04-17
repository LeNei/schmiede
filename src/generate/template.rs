use askama::Template;

use super::crud::CrudOperations;
use super::data_types::IDType;

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

/*
#[derive(Template)]
#[template(path = "page.rs.templ", escape = "html")]
pub struct PageTemplate<'a> {
    pub function_name: &'a str,
    pub model_name: &'a str,
    pub route: &'a str,
}
*/

#[derive(Template)]
#[template(path = "api.rs.templ", escape = "html")]
pub struct ApiTemplate<'a> {
    pub name: &'a str,
    pub struct_name: &'a str,
    pub crud_operations: CrudOperations,
}
