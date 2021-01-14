use crate::uuid::model::{IdAccessible, UuidError};
use anyhow::Result;
use async_trait::async_trait;
use database_layer_actix::{format_alias, format_datetime};
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

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

#[async_trait]
impl IdAccessible for Page {
    async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Self, UuidError> {
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
        .fetch_one(pool);
        let revisions_fut = sqlx::query!(
            r#"SELECT id, date FROM page_revision WHERE page_repository_id = ?"#,
            id
        )
        .fetch_all(pool);
        let (page, revisions) = try_join!(page_fut, revisions_fut)?;

        if revisions.is_empty() {
            return Err(UuidError::NotFound);
        } else {
            Ok(Page {
                __typename: String::from("Page"),
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
