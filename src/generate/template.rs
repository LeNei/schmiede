use askama::Template;

use crate::config::DatabaseDriver;

use super::crud::CrudOperations;
use super::data_types::IDType;
use super::exporters::Export;

#[derive(Template)]
#[template(path = "generate/db/sqlx/up.sql.templ", escape = "html")]
pub struct SqlxUpTemplate<'a> {
    pub name: &'a str,
    pub rows: Vec<String>,
    pub id: IDType,
}

#[derive(Template)]
#[template(path = "generate/db/sqlx/down.sql.templ", escape = "html")]
pub struct SqlxDownTemplate<'a> {
    pub name: &'a str,
}

#[derive(Template)]
#[template(path = "generate/db/diesel/up.sql.templ", escape = "html")]
pub struct DieselUpTemplate<'a> {
    pub name: &'a str,
    pub rows: Vec<String>,
    pub id: IDType,
}

#[derive(Template)]
#[template(path = "generate/db/diesel/down.sql.templ", escape = "html")]
pub struct DieselDownTemplate<'a> {
    pub name: &'a str,
}

pub fn get_db_template<'a>(
    name: &'a str,
    rows: Vec<String>,
    id: IDType,
    database_driver: &DatabaseDriver,
) -> Vec<Box<dyn Export + 'a>> {
    match database_driver {
        DatabaseDriver::Sqlx => {
            vec![
                Box::new(SqlxUpTemplate { name, rows, id }),
                Box::new(SqlxDownTemplate { name }),
            ]
        }
        DatabaseDriver::Diesel => {
            vec![
                Box::new(DieselUpTemplate { name, rows, id }),
                Box::new(DieselDownTemplate { name }),
            ]
        }
    }
}

#[derive(Template)]
#[template(path = "generate/models/sqlx.rs.templ", escape = "html")]
pub struct SqlxModelTemplate<'a> {
    pub id: IDType,
    pub name: &'a str,
    pub struct_name: &'a str,
    pub rows: Vec<String>,
}

#[derive(Template)]
#[template(path = "generate/models/diesel.rs.templ", escape = "html")]
pub struct DieselModelTemplate<'a> {
    pub id: IDType,
    pub name: &'a str,
    pub struct_name: &'a str,
    pub rows: Vec<String>,
}

pub fn get_model_template<'a>(
    name: &'a str,
    struct_name: &'a str,
    id: IDType,
    rows: Vec<String>,
    database_driver: DatabaseDriver,
) -> Box<dyn Export + 'a> {
    match database_driver {
        DatabaseDriver::Sqlx => Box::new(SqlxModelTemplate {
            id,
            name,
            struct_name,
            rows,
        }),
        DatabaseDriver::Diesel => Box::new(DieselModelTemplate {
            id,
            name,
            struct_name,
            rows,
        }),
    }
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
#[template(path = "generate/api/axum_diesel.rs.templ", escape = "html")]
pub struct AxumDieselTemplate<'a> {
    pub name: &'a str,
    pub struct_name: &'a str,
    pub crud_operations: CrudOperations,
}

#[derive(Template)]
#[template(path = "generate/api/axum_sqlx.rs.templ", escape = "html")]
pub struct AxumSqlxTemplate<'a> {
    pub name: &'a str,
    pub struct_name: &'a str,
    pub crud_operations: CrudOperations,
}

pub fn get_api_template<'a>(
    name: &'a str,
    struct_name: &'a str,
    crud_operations: CrudOperations,
    database_driver: DatabaseDriver,
) -> Box<dyn Export + 'a> {
    match database_driver {
        DatabaseDriver::Sqlx => Box::new(AxumSqlxTemplate {
            name,
            struct_name,
            crud_operations,
        }),
        DatabaseDriver::Diesel => Box::new(AxumDieselTemplate {
            name,
            struct_name,
            crud_operations,
        }),
    }
}
