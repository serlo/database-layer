use async_trait::async_trait;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::format_alias;

pub struct Attachment {}

#[async_trait]
impl UuidFetcher for Attachment {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        Self::fetch_via_transaction(id, pool).await
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
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
        .map(|attachment| Uuid {
            id,
            trashed: attachment.trashed != 0,
            alias: format_alias(
                Self::get_context().as_deref(),
                id,
                Some(attachment.name.as_str()),
            ),
            concrete_uuid: ConcreteUuid::Attachment,
        })
    }
}

impl Attachment {
    pub fn get_context() -> Option<String> {
        Some("attachment".to_string())
    }
}
