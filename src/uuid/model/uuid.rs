use serde::Serialize;
use sqlx::MySqlPool;

use crate::uuid::model::attachment::Attachment;
use crate::uuid::model::blog_post::BlogPost;
use crate::uuid::model::comment::Comment;
use crate::uuid::model::entity::Entity;
use crate::uuid::model::entity_revision::EntityRevision;
use crate::uuid::model::page::Page;
use crate::uuid::model::page_revision::PageRevision;
use crate::uuid::model::taxonomy_term::TaxonomyTerm;
use crate::uuid::model::user::User;
use crate::uuid::model::{IdAccessible, ObjectType, UuidError};

//FIXME: I think the name Uuid is misleading,
// since the object does not represent a uuid,
// but a resource identified by a uuid.
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

impl std::str::FromStr for ObjectType {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.into())).map_err(|_e| {
            UuidError::InvalidDiscriminator {
                discriminator: s.into(),
            }
        })
    }
}

impl Uuid {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await?;

        match uuid.discriminator.as_str().parse()? {
            ObjectType::Attachment => Ok(Uuid::Attachment(Attachment::find_by_id(id, pool).await?)),
            ObjectType::BlogPost => Ok(Uuid::BlogPost(BlogPost::find_by_id(id, pool).await?)),
            ObjectType::Comment => Ok(Uuid::Comment(Comment::find_by_id(id, pool).await?)),
            ObjectType::Entity => Ok(Uuid::Entity(Entity::find_by_id(id, pool).await?)),
            ObjectType::EntityRevision => Ok(Uuid::EntityRevision(
                EntityRevision::find_by_id(id, pool).await?,
            )),
            ObjectType::Page => Ok(Uuid::Page(Page::find_by_id(id, pool).await?)),
            ObjectType::PageRevision => Ok(Uuid::PageRevision(
                PageRevision::find_by_id(id, pool).await?,
            )),
            ObjectType::TaxonomyTerm => Ok(Uuid::TaxonomyTerm(
                TaxonomyTerm::find_by_id(id, pool).await?,
            )),
            ObjectType::User => Ok(Uuid::User(User::find_by_id(id, pool).await?)),
        }
    }

    pub async fn find_context_by_id(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<String>, UuidError> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await?;

        match uuid.discriminator.as_str().parse()? {
            ObjectType::Attachment => Ok(Attachment::get_context()),
            ObjectType::BlogPost => Ok(BlogPost::get_context()),
            // This is done intentionally to avoid a recursive `async fn` and because this is not needed.
            //FIXME: what is done intentionally?
            ObjectType::Comment => Ok(None),
            ObjectType::Entity => Entity::find_canonical_subject_by_id(id, pool).await,
            ObjectType::EntityRevision => {
                EntityRevision::find_canonical_subject_by_id(id, pool).await
            }
            ObjectType::Page => Ok(None),         // TODO:
            ObjectType::PageRevision => Ok(None), // TODO:
            ObjectType::TaxonomyTerm => TaxonomyTerm::find_canonical_subject_by_id(id, pool).await,
            ObjectType::User => Ok(User::get_context()),
        }
    }
}
