use anyhow::Result;
use chrono::{DateTime, TimeZone};
use serde::Serialize;
use sqlx::{FromRow, MySqlPool};

pub struct Uuid {}

/// Represents a User data record
#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub __typename: String,
    pub username: String,
    pub date: String,
    pub last_login: Option<String>,
    pub description: Option<String>,
}

impl Uuid {
    // TODO: We'd like an union type here (e.g. returns on of the concrete uuid types). Not entirely sure how to do this in a idiomatic way.
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<User> {
        let row = sqlx::query!(
            r#"
                SELECT * FROM uuid WHERE id = ?
            "#,
            id
        )
        .fetch_one(&*pool)
        .await?;

        match row.discriminator.as_str() {
            "user" => {
                let user = sqlx::query!(
                    r#"
                SELECT * FROM user WHERE id = ?
            "#,
                    id
                )
                .fetch_one(&*pool)
                .await?;
                Ok({
                    User {
                        id: row.id as i32,
                        trashed: row.trashed != 0,
                        alias: format!("/user/{}/{}", row.id, user.username),
                        __typename: String::from("User"),
                        username: user.username,
                        date: format_datetime(&user.date),
                        last_login: user.last_login.map(|date| format_datetime(&date)),
                        description: user.description,
                    }
                })
            }
            _ => {
                panic!("TODO")
            }
        }
    }
}

pub fn format_datetime<Tz: TimeZone>(datetime: &DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    // The datetime in database is persisted as UTC but is actually in local time. So we reinterpreted it here.
    let naive_datetime = datetime.naive_utc();
    chrono_tz::Europe::Berlin
        .from_local_datetime(&naive_datetime)
        .unwrap()
        .to_rfc3339()
}
