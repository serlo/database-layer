use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use super::{
    attachment::Attachment, blog_post::BlogPost, comment::Comment, entity::Entity,
    entity_revision::EntityRevision, page::Page, page_revision::PageRevision,
    taxonomy_term::TaxonomyTerm, user::User,
};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Uuid {
    Attachment(Attachment),
    BlogPost(BlogPost),
    Comment(Comment),
    Entity(Entity),
    EntityRevision(EntityRevision),
    Page(Page),
    PageRevision(PageRevision),
    TaxonomyTerm(TaxonomyTerm),
    User(User),
}

impl Uuid {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => UuidError::NotFound,
                inner => UuidError::DatabaseError { inner },
            })?;
        match uuid.discriminator.as_str() {
            "attachment" => Ok(Uuid::Attachment(Attachment::fetch(id, pool).await?)),
            "blogPost" => Ok(Uuid::BlogPost(BlogPost::fetch(id, pool).await?)),
            "comment" => Ok(Uuid::Comment(Comment::fetch(id, pool).await?)),
            "entity" => Ok(Uuid::Entity(Entity::fetch(id, pool).await?)),
            "entityRevision" => Ok(Uuid::EntityRevision(EntityRevision::fetch(id, pool).await?)),
            "page" => Ok(Uuid::Page(Page::fetch(id, pool).await?)),
            "pageRevision" => Ok(Uuid::PageRevision(PageRevision::fetch(id, pool).await?)),
            "taxonomyTerm" => Ok(Uuid::TaxonomyTerm(TaxonomyTerm::fetch(id, pool).await?)),
            "user" => Ok(Uuid::User(User::fetch(id, pool).await?)),
            _ => Err(UuidError::UnsupportedDiscriminator {
                discriminator: uuid.discriminator,
            }),
        }
    }

    pub async fn fetch_context(id: i32, pool: &MySqlPool) -> Result<Option<String>, UuidError> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => UuidError::NotFound,
                inner => UuidError::DatabaseError { inner },
            })?;
        let context = match uuid.discriminator.as_str() {
            "attachment" => Ok(Attachment::get_context()),
            "blogPost" => Ok(BlogPost::get_context()),
            // This is done intentionally to avoid a recursive `async fn` and because this is not needed.
            "comment" => Ok(None),
            "entity" => Entity::fetch_canonical_subject(id, pool).await,
            "entityRevision" => EntityRevision::fetch_canonical_subject(id, pool).await,
            "page" => Ok(None),         // TODO:
            "pageRevision" => Ok(None), // TODO:
            "taxonomyTerm" => TaxonomyTerm::fetch_canonical_subject(id, pool).await,
            "user" => Ok(User::get_context()),
            _ => Ok(None),
        };
        context.map_err(|inner| UuidError::DatabaseError { inner })
    }

    pub fn get_alias(&self) -> String {
        match self {
            Uuid::Attachment(attachment) => attachment.alias.to_string(),
            Uuid::BlogPost(blog) => blog.alias.to_string(),
            Uuid::Comment(comment) => comment.alias.to_string(),
            Uuid::Entity(entity) => entity.abstract_entity.alias.to_string(),
            Uuid::EntityRevision(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
            Uuid::Page(page) => page.alias.to_string(),
            Uuid::PageRevision(page_revision) => page_revision.alias.to_string(),
            Uuid::TaxonomyTerm(taxonomy_term) => taxonomy_term.alias.to_string(),
            Uuid::User(user) => user.alias.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum UuidError {
    #[error("UUID cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error(
        "UUID cannot be fetched because its discriminator `{discriminator:?}` is not supported."
    )]
    UnsupportedDiscriminator { discriminator: String },
    #[error("Entity cannot be fetched because its type `{name:?}` is not supported.")]
    UnsupportedEntityType { name: String },
    #[error("EntityRevision cannot be fetched because its type `{name:?}` is not supported.")]
    UnsupportedEntityRevisionType { name: String },
    #[error("Entity cannot be fetched because its parent is missing.")]
    EntityMissingRequiredParent,
    #[error("UUID cannot be fetched because it does not exist.")]
    NotFound,
}

impl From<sqlx::Error> for UuidError {
    fn from(inner: sqlx::Error) -> Self {
        UuidError::DatabaseError { inner }
    }
}
