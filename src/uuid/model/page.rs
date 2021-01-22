use serde::Serialize;
use sqlx::MySqlPool;

use crate::{format_alias, format_datetime};

use super::UuidError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub instance: String,
    pub current_revision_id: Option<i32>,
    pub revision_ids: Vec<i32>,
    pub date: String,
    pub license_id: i32,
}

impl Page {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Page, UuidError> {
        let page = sqlx::query!(
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
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            inner => UuidError::DatabaseError { inner },
        })?;

        let revisions = sqlx::query!(
            r#"SELECT id, date FROM page_revision WHERE page_repository_id = ?"#,
            id
        )
        .fetch_all(pool)
        .await
        .map_err(|inner| UuidError::DatabaseError { inner })?;

        if revisions.is_empty() {
            Err(UuidError::NotFound)
        } else {
            Ok(Page {
                __typename: "Page".to_string(),
                id,
                trashed: page.trashed != 0,
                // TODO:
                alias: format_alias(None, id, page.title.as_deref()),
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
}
