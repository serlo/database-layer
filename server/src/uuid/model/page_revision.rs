use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::datetime::DateTime;

#[derive(Debug, Serialize)]
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

#[async_trait]
impl UuidFetcher for PageRevision {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        Self::fetch_via_transaction(id, pool).await
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        sqlx::query!(
            r#"
                SELECT u.trashed, r.title, r.content, r.date, r.author_id, r.page_repository_id
                    FROM page_revision r
                    JOIN uuid u ON u.id = r.id
                    WHERE r.id = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })
        .map(|revision| Uuid {
            id,
            trashed: revision.trashed != 0,
            alias: format!("/entity/repository/compare/0/{id}"),
            concrete_uuid: ConcreteUuid::PageRevision(PageRevision {
                __typename: "PageRevision".to_string(),
                title: revision.title,
                content: revision.content,
                date: revision.date.into(),
                author_id: revision.author_id as i32,
                repository_id: revision.page_repository_id as i32,
            }),
        })
    }
}
