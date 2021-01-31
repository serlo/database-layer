use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::UuidError;
use crate::database::Executor;
use crate::format_alias;
use crate::uuid::model::uuid::UuidFetcher;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
}

#[async_trait]
impl UuidFetcher for Attachment {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Attachment, UuidError> {
        Self::fetch_via_transaction(id, pool).await
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Self, UuidError>
    where
        E: Executor<'a>,
    {
        sqlx::query!(
            r#"
                SELECT u.trashed, f.name
                    FROM attachment_file f
                    JOIN attachment_container c ON c.id = f.attachment_id
                    JOIN uuid u ON u.id = c.id
                    WHERE c.id = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })
        .map(|attachment| Attachment {
            id,
            trashed: attachment.trashed != 0,
            alias: format_alias(
                Self::get_context().as_deref(),
                id,
                Some(attachment.name.as_str()),
            ),
        })
    }
}

impl Attachment {
    pub fn get_context() -> Option<String> {
        Some("attachment".to_string())
    }
}
