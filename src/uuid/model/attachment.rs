use serde::Serialize;
use sqlx::MySqlPool;

use super::UuidError;
use crate::format_alias;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
}

impl Attachment {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Attachment, UuidError> {
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
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            inner => UuidError::DatabaseError { inner },
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

    pub fn get_context() -> Option<String> {
        Some("attachment".to_string())
    }
}
