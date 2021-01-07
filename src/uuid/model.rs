use anyhow::Result;
use chrono::{DateTime, TimeZone};
use futures::try_join;
use regex::Regex;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Uuid {
    User(User),
    Page(Page),
}

impl Uuid {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Uuid> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(&*pool)
            .await?;
        match uuid.discriminator.as_str() {
            "page" => Ok(Uuid::Page(Page::find_by_id(id, pool).await?)),
            "user" => Ok(Uuid::User(User::find_by_id(id, pool).await?)),
            _ => {
                panic!("TODO")
            }
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    #[serde(rename(serialize = "__typename"))]
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
            alias: format!("/user/{}/{}", id, user.username),
            username: user.username,
            date: format_datetime(&user.date),
            last_login: user.last_login.map(|date| format_datetime(&date)),
            description: user.description,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    #[serde(rename(serialize = "__typename"))]
    pub instance: String,
    pub current_revision_id: Option<i32>,
    pub revision_ids: Vec<i32>,
    pub date: String,
    pub license_id: i32,
}

impl Page {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Page> {
        let page_fut = sqlx::query!(
            r#"
                SELECT u.trashed, i.subdomain, p.current_revision_id, p.license_id, r.title
                    FROM page_repository p
                    JOIN uuid u ON u.id = p.id
                    JOIN instance i ON i.id = p.instance_id
                    LEFT JOIN page_revision r ON r.id = p.current_revision_id
                    WHERE p.id = ?
            "#,
            id
        )
        .fetch_one(&*pool);
        let revisions_fut = sqlx::query!(
            r#"SELECT id, date FROM page_revision WHERE page_repository_id = ?"#,
            id
        )
        .fetch_all(&*pool);
        let (page, revisions) = try_join!(page_fut, revisions_fut)?;
        Ok(Page {
            id,
            trashed: page.trashed != 0,
            // TODO:
            alias: format_alias(None, id, page.title.as_deref()),
            __typename: String::from("Page"),
            instance: page.subdomain,
            current_revision_id: page.current_revision_id,
            revision_ids: revisions
                .iter()
                .rev()
                .map(|revision| revision.id as i32)
                .collect(),
            date: format_datetime(&revisions[0].date),
            license_id: page.license_id,
        })
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

pub fn format_alias(prefix: Option<&str>, id: i32, suffix: Option<&str>) -> String {
    let prefix = prefix
        .map(|p| format!("/{}", slugify(p)))
        .unwrap_or_else(|| String::from(""));
    let suffix = suffix.map(slugify).unwrap_or_else(|| String::from(""));
    format!("{}/{}/{}", prefix, id, suffix)
}

pub fn slugify(segment: &str) -> String {
    let segment = Regex::new(r#"['"`=+*&^%$#@!<>?]"#)
        .unwrap()
        .replace_all(&segment, "");
    let segment = Regex::new(r"[\[\]{}() ,;/|]+")
        .unwrap()
        .replace_all(&segment, "-");
    String::from(segment.to_lowercase().trim_matches('-'))
}
