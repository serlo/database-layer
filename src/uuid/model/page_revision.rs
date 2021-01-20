use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::{format_alias, format_datetime};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRevision {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub title: String,
    pub content: String,
    pub date: String,
    pub author_id: i32,
    pub repository_id: i32,
}

impl PageRevision {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<PageRevision> {
        let revision = sqlx::query!(
            r#"
                SELECT u.trashed, r.title, r.content, r.date, r.author_id, r.page_repository_id
                    FROM page_revision r
                    JOIN uuid u ON u.id = r.id
                    WHERE r.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(PageRevision {
            __typename: "PageRevision".to_string(),
            id,
            trashed: revision.trashed != 0,
            // TODO:
            alias: format_alias(None, id, Some(&revision.title)),
            title: revision.title,
            content: revision.content,
            date: format_datetime(&revision.date),
            author_id: revision.author_id as i32,
            repository_id: revision.page_repository_id as i32,
        })
    }
}
