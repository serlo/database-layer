use crate::uuid::model::{IdAccessible, UuidError};
use async_trait::async_trait;
use database_layer_actix::format_alias;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogPost {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
}

#[async_trait]
impl IdAccessible for BlogPost {
    async fn find_by_id(id: i32, pool: &sqlx::MySqlPool) -> Result<Self, UuidError> {
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
}

impl BlogPost {
    pub fn get_context() -> Option<String> {
        return Some(String::from("blog"));
    }
}
