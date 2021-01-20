use anyhow::Result;
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
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Uuid> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => anyhow::Error::new(UuidError::NotFound { id }),
                e => anyhow::Error::new(e),
            })?;
        match uuid.discriminator.as_str() {
            "attachment" => Ok(Uuid::Attachment(Attachment::find_by_id(id, pool).await?)),
            "blogPost" => Ok(Uuid::BlogPost(BlogPost::find_by_id(id, pool).await?)),
            "comment" => Ok(Uuid::Comment(Comment::find_by_id(id, pool).await?)),
            "entity" => Ok(Uuid::Entity(Entity::find_by_id(id, pool).await?)),
            "entityRevision" => Ok(Uuid::EntityRevision(
                EntityRevision::find_by_id(id, pool).await?,
            )),
            "page" => Ok(Uuid::Page(Page::find_by_id(id, pool).await?)),
            "pageRevision" => Ok(Uuid::PageRevision(
                PageRevision::find_by_id(id, pool).await?,
            )),
            "taxonomyTerm" => Ok(Uuid::TaxonomyTerm(
                TaxonomyTerm::find_by_id(id, pool).await?,
            )),
            "user" => Ok(Uuid::User(User::find_by_id(id, pool).await?)),
            _ => Err(anyhow::Error::new(UuidError::UnsupportedDiscriminator {
                id,
                discriminator: uuid.discriminator,
            })),
        }
    }

    pub async fn find_context_by_id(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<String>, sqlx::Error> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await?;
        match uuid.discriminator.as_str() {
            "attachment" => Ok(Attachment::get_context()),
            "blogPost" => Ok(BlogPost::get_context()),
            // This is done intentionally to avoid a recursive `async fn` and because this is not needed.
            "comment" => Ok(None),
            "entity" => Entity::find_canonical_subject_by_id(id, pool).await,
            "entityRevision" => EntityRevision::find_canonical_subject_by_id(id, pool).await,
            "page" => Ok(None),         // TODO:
            "pageRevision" => Ok(None), // TODO:
            "taxonomyTerm" => TaxonomyTerm::find_canonical_subject_by_id(id, pool).await,
            "user" => Ok(User::get_context()),
            _ => Ok(None),
        }
    }

    pub fn get_alias(&self) -> String {
        match self {
            Uuid::Attachment(attachment) => attachment.alias.to_string(),
            Uuid::BlogPost(blog) => blog.alias.to_string(),
            Uuid::Comment(comment) => comment.alias.to_string(),
            Uuid::Entity(entity) => entity.get_alias(),
            Uuid::EntityRevision(entity_revision) => entity_revision.alias.to_string(),
            Uuid::Page(page) => page.alias.to_string(),
            Uuid::PageRevision(page_revision) => page_revision.alias.to_string(),
            Uuid::TaxonomyTerm(taxonomy_term) => taxonomy_term.alias.to_string(),
            Uuid::User(user) => user.alias.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum UuidError {
    #[error("UUID {id:?} can't be fetched because is `{discriminator:?}` is not supported.")]
    UnsupportedDiscriminator { id: i32, discriminator: String },
    #[error("UUID {id:?} not found.")]
    NotFound { id: i32 },
}
