use async_trait::async_trait;

use serde::Serialize;


use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::datetime::DateTime;
use crate::format_alias;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub username: String,
    pub date: DateTime,
    pub last_login: Option<DateTime>,
    pub description: Option<String>,
    pub roles: Vec<String>,
}

macro_rules! fetch_one_user {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT trashed, username, date, last_login, description
                    FROM user
                    JOIN uuid ON user.id = uuid.id
                    WHERE user.id = ?
            "#,
            $id
        )
        .fetch_one($executor)
    };
}

macro_rules! fetch_all_roles {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT r.name
                    FROM role r
                    JOIN role_user ru on r.id = ru.role_id
                    WHERE ru.user_id = ?
            "#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! to_user {
    ($id: expr, $user: expr, $roles: expr) => {{
        let user = $user.map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let roles = $roles?;

        Ok(Uuid {
            id: $id,
            trashed: user.trashed != 0,
            alias: format_alias(Self::get_context().as_deref(), $id, Some(&user.username)),
            concrete_uuid: ConcreteUuid::User(User {
                __typename: "User".to_string(),
                username: user.username,
                date: user.date.into(),
                last_login: user.last_login.map(|date| date.into()),
                description: user.description,
                roles: roles.iter().map(|role| role.name.to_string()).collect(),
            }),
        })
    }};
}

#[async_trait]
impl UuidFetcher for User {
    async fn fetch<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql> + std::marker::Send>(
        id: i32,
        acquire_from: A,
    ) -> Result<Uuid, UuidError> {
        let mut transaction = acquire_from.begin().await?;
        let user = fetch_one_user!(id, &mut *transaction).await;
        let roles = fetch_all_roles!(id, &mut *transaction).await;
        transaction.commit().await?;
        to_user!(id, user, roles)
    }
}

impl User {
    pub fn get_context() -> Option<String> {
        Some("user".to_string())
    }
}
