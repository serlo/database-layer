use crate::uuid::model::{IdAccessible, UuidError};
use async_trait::async_trait;
use database_layer_actix::{format_alias, format_datetime};
use serde::Serialize;

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

#[async_trait]
impl IdAccessible for PageRevision {
    async fn find_by_id(id: i32, pool: &sqlx::MySqlPool) -> Result<Self, UuidError> {
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
            __typename: String::from("PageRevision"),
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
