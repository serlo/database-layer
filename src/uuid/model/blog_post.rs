use async_trait::async_trait;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::format_alias;

pub struct BlogPost {}

#[async_trait]
impl UuidFetcher for BlogPost {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        Self::fetch_via_transaction(id, pool).await
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        sqlx::query!(
            r#"
                SELECT u.trashed, b.title
                    FROM blog_post b
                    JOIN uuid u ON u.id = b.id
                    WHERE b.id = ?
            "#,
            id
        )
        .fetch_one(executor)
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
