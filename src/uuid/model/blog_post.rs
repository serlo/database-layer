use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::format_alias;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogPost {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
}

impl BlogPost {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<BlogPost> {
        let blog = sqlx::query!(
            r#"
                SELECT u.trashed, b.title
                    FROM blog_post b
                    JOIN uuid u ON u.id = b.id
                    WHERE b.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(BlogPost {
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
