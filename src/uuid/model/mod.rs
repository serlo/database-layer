mod attachment;
mod blog_post;
mod comment;
mod entity;
mod entity_revision;
mod page;
mod page_revision;
mod taxonomy_term;
mod user;
mod uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use self::uuid::Uuid;
use async_trait::async_trait;

#[derive(Error, Debug)]
pub enum UuidError {
    #[error("`Database error {inner:?}` occured.")]
    DatabaseError { inner: sqlx::Error },
    #[error("`{discriminator:?}` is not a valid uuid type.")]
    InvalidDiscriminator { discriminator: String },
    #[error("UUID not found in database.")]
    NotFound,
}

impl From<sqlx::Error> for UuidError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            e => UuidError::DatabaseError { inner: e },
        }
    }
}

/// Interface for accessing resources that can be identified
/// Through a UUID.
#[async_trait]
trait IdAccessible {
    async fn find_by_id(id: i32, pool: &sqlx::MySqlPool) -> Result<Self, UuidError>
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ObjectType {
    Attachment,
    BlogPost,
    Comment,
    Entity,
    EntityRevision,
    Page,
    PageRevision,
    TaxonomyTerm,
    User,
}
