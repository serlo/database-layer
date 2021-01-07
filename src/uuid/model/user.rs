use anyhow::Result;
use database_layer_actix::{format_alias, format_datetime};
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub username: String,
    pub date: String,
    pub last_login: Option<String>,
    pub description: Option<String>,
}

impl User {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<User> {
        let user = sqlx::query!(
            r#"
                SELECT trashed, username, date, last_login, description
                    FROM user
                    JOIN uuid ON user.id = uuid.id
                    WHERE user.id = ?
            "#,
            id
        )
        .fetch_one(&*pool)
        .await?;
        Ok(User {
            __typename: String::from("User"),
            id,
            trashed: user.trashed != 0,
            alias: format_alias(Some("user"), id, Some(&user.username)),
            username: user.username,
            date: format_datetime(&user.date),
            last_login: user.last_login.map(|date| format_datetime(&date)),
            description: user.description,
        })
    }
}
