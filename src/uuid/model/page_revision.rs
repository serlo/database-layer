use serde::Serialize;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError};
use crate::datetime::DateTime;
use crate::format_alias;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRevision {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub title: String,
    pub content: String,
    pub date: DateTime,
    pub author_id: i32,
    pub repository_id: i32,
}

impl PageRevision {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        sqlx::query!(
            r#"
                SELECT u.trashed, r.title, r.content, r.date, r.author_id, r.page_repository_id
                    FROM page_revision r
                    JOIN uuid u ON u.id = r.id
                    WHERE r.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })
        .map(|revision| {
            Uuid {
                id,
                trashed: revision.trashed != 0,
                // TODO:
                alias: format_alias(None, id, Some(&revision.title)),
                concrete_uuid: ConcreteUuid::PageRevision(PageRevision {
                    __typename: "PageRevision".to_string(),
                    title: revision.title,
                    content: revision.content,
                    date: revision.date.into(),
                    author_id: revision.author_id as i32,
                    repository_id: revision.page_repository_id as i32,
                }),
            }
        })
    }
}
