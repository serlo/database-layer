use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use crate::uuid::model::attachment::Attachment;
use crate::uuid::model::blog_post::BlogPost;
use crate::uuid::model::comment::Comment;
use crate::uuid::model::entity::Entity;
use crate::uuid::model::entity_revision::EntityRevision;
use crate::uuid::model::page::Page;
use crate::uuid::model::page_revision::PageRevision;
use crate::uuid::model::taxonomy_term::TaxonomyTerm;
use crate::uuid::model::user::User;

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
}

#[derive(Error, Debug)]
pub enum UuidError {
    #[error("UUID {id:?} can't be fetched because is `{discriminator:?}` is not supported.")]
    UnsupportedDiscriminator { id: i32, discriminator: String },
    #[error("UUID {id:?} not found.")]
    NotFound { id: i32 },
}
