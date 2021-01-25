use serde::Serialize;
use sqlx::MySqlPool;

use super::UuidError;
use crate::datetime::DateTime;
use crate::format_alias;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub username: String,
    pub date: DateTime,
    pub last_login: Option<DateTime>,
    pub description: Option<String>,
}

impl User {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<User, UuidError> {
        sqlx::query!(
            r#"
                SELECT trashed, username, date, last_login, description
                    FROM user
                    JOIN uuid ON user.id = uuid.id
                    WHERE user.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            inner => UuidError::DatabaseError { inner },
        })
        .map(|user| User {
            __typename: "User".to_string(),
            id,
            trashed: user.trashed != 0,
            alias: format_alias(Self::get_context().as_deref(), id, Some(&user.username)),
            username: user.username,
            date: user.date.into(),
            last_login: user.last_login.map(|date| date.into()),
            description: user.description,
        })
    }

    pub fn get_context() -> Option<String> {
        Some("user".to_string())
    }
}
