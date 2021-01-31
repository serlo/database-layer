use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::format_alias;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub username: String,
    pub date: DateTime,
    pub last_login: Option<DateTime>,
    pub description: Option<String>,
}

#[async_trait]
impl UuidFetcher for User {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        Self::fetch_via_transaction(id, pool).await
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        sqlx::query!(
            r#"
                SELECT trashed, username, date, last_login, description
                    FROM user
                    JOIN uuid ON user.id = uuid.id
                    WHERE user.id = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })
        .map(|user| Uuid {
            id,
            trashed: user.trashed != 0,
            alias: format_alias(Self::get_context().as_deref(), id, Some(&user.username)),
            concrete_uuid: ConcreteUuid::User(User {
                __typename: "User".to_string(),
                username: user.username,
                date: user.date.into(),
                last_login: user.last_login.map(|date| date.into()),
                description: user.description,
            }),
        })
    }
}

impl User {
    pub fn get_context() -> Option<String> {
        Some("user".to_string())
    }
}
