use chrono::prelude::*;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Serialize, Deserialize };

#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Deserialize, Serialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::UserRole"]
pub enum UserRole {
    Admin,
    User,
}

impl UserRole {
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }
}

#[derive(Selectable, Queryable, Insertable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub verified: bool,
    pub password: String,
    pub verification_token: Option<String>,
    pub token_expires_at: Option<NaiveDateTime>,
    pub role: UserRole,
    #[serde(rename = "createdAt")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}