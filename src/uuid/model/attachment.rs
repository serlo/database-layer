use anyhow::Result;
use serde::Serialize;
use serlo_org_database_layer::format_alias;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
}

impl Attachment {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Attachment> {
        let attachment = sqlx::query!(
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
        .await?;
        Ok(Attachment {
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
        Some(String::from("attachment"))
    }
}
