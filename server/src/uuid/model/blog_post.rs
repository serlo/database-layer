use async_trait::async_trait;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::format_alias;

pub struct BlogPost {}

#[async_trait]
impl UuidFetcher for BlogPost {
    async fn fetch<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql> + std::marker::Send>(
        id: i32,
        acquire_from: A,
    ) -> Result<Uuid, UuidError> {
        let mut connection = acquire_from.acquire().await?;
        sqlx::query!(
            r#"
                SELECT u.trashed, b.title
                    FROM blog_post b
                    JOIN uuid u ON u.id = b.id
                    WHERE b.id = ?
            "#,
            id
        )
        .fetch_one(&mut *connection)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })
        .map(|blog| Uuid {
            id,
            trashed: blog.trashed != 0,
            alias: format_alias(
                Self::get_context().as_deref(),
                id,
                Some(blog.title.as_str()),
            ),
            concrete_uuid: ConcreteUuid::BlogPost,
        })
    }
}

impl BlogPost {
    pub fn get_context() -> Option<String> {
        Some("blog".to_string())
    }
}
