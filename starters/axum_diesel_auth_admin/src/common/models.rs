use super::traits::Displayable;
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Clone, Debug, Displayable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(Pg))]
pub struct Users {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
    #[serde(skip_serializing)]
    pub password: &'a str,
    pub role: &'a str,
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserSchema {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Queryable, Selectable, Serialize, Clone, Debug, Displayable)]
#[diesel(table_name = crate::schema::post)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: uuid::Uuid,
    title: String,
    content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Clone, Debug)]
#[diesel(table_name = crate::schema::post)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPost {
    title: String,
    content: String,
}
