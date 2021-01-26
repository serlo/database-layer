use serde::Serialize;
use sqlx::MySqlPool;

use super::UuidError;
use crate::format_alias;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogPost {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
}

impl BlogPost {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<BlogPost, UuidError> {
        sqlx::query!(
            r#"
                SELECT u.trashed, b.title
                    FROM blog_post b
                    JOIN uuid u ON u.id = b.id
                    WHERE b.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })
        .map(|blog| BlogPost {
            id,
            trashed: blog.trashed != 0,
            alias: format_alias(
                Self::get_context().as_deref(),
                id,
                Some(blog.title.as_str()),
            ),
        })
    }

    pub fn get_context() -> Option<String> {
        Some("blog".to_string())
    }
}
