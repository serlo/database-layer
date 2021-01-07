use anyhow::Result;
use chrono::{DateTime, TimeZone};
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Uuid {
    User(User),
    Page(Page),
}

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub __typename: String,
    pub instance: String,
    pub current_revision_id: Option<i32>,
    // pub revision_ids: Vec<i32>,
    // pub date: String,
    pub license_id: i32,
}

impl Uuid {
    // TODO: We'd like an union type here (e.g. returns one of the concrete uuid types). Not entirely sure how to do this in a idiomatic way.
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Uuid> {
        let uuid = sqlx::query!(r#"SELECT * FROM uuid WHERE id = ?"#, id)
            .fetch_one(&*pool)
            .await?;

        match uuid.discriminator.as_str() {
            "page" => {
                // TODO:
                let title = "";
                let page = sqlx::query!(r#"SELECT i.subdomain, pr.current_revision_id, pr.license_id FROM page_repository pr JOIN instance i ON pr.instance_id = i.id WHERE pr.id = ?"#, id)
                    .fetch_one(&*pool)
                    .await?;
                Ok(Uuid::Page(Page {
                    id: uuid.id as i32,
                    trashed: uuid.trashed != 0,
                    // TODO:
                    alias: format!("/{}/{}", uuid.id, title),
                    __typename: String::from("Page"),
                    instance: page.subdomain,
                    current_revision_id: page.current_revision_id,
                    license_id: page.license_id,
                }))
            }
            "user" => {
                let user = sqlx::query!(r#"SELECT * FROM user WHERE id = ?"#, id)
                    .fetch_one(&*pool)
                    .await?;
                Ok(Uuid::User(User {
                    id: uuid.id as i32,
                    trashed: uuid.trashed != 0,
                    alias: format!("/user/{}/{}", uuid.id, user.username),
                    __typename: String::from("User"),
                    username: user.username,
                    date: format_datetime(&user.date),
                    last_login: user.last_login.map(|date| format_datetime(&date)),
                    description: user.description,
                }))
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
