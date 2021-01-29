use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;
use crate::event::{EventError, SetUuidStateEventPayload};

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

#[derive(Error, Debug)]
pub enum UuidError {
    #[error("UUID cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("UUID cannot be fetched because its instance is invalid.")]
    InvalidInstance,
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

impl Uuid {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => UuidError::NotFound,
                error => error.into(),
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
                error => error.into(),
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

        Ok(context?)
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidStatePayload {
    ids: Vec<i32>,
    user_id: i32,
    trashed: bool,
}

#[derive(Error, Debug)]
pub enum SetUuidStateError {
    #[error("UUID state cannot be set because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("UUID state cannot be set because of an internal error: {inner:?}.")]
    EventError { inner: EventError },
}

impl From<sqlx::Error> for SetUuidStateError {
    fn from(inner: sqlx::Error) -> Self {
        SetUuidStateError::DatabaseError { inner }
    }
}

impl From<EventError> for SetUuidStateError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => SetUuidStateError::EventError { inner },
        }
    }
}

impl Uuid {
    pub async fn set_uuid_state<'a, E>(
        payload: SetUuidStatePayload,
        executor: E,
    ) -> Result<(), SetUuidStateError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for id in payload.ids.into_iter() {
            let result = sqlx::query!(
                r#"
                    SELECT trashed FROM uuid WHERE id = ? AND discriminator != 'user'
                "#,
                id
            )
            .fetch_one(&mut transaction)
            .await;

            match result {
                Ok(uuid) => {
                    // UUID has already the correct state, skip
                    if (uuid.trashed != 0) == payload.trashed {
                        continue;
                    }
                }
                Err(sqlx::Error::RowNotFound) => {
                    // UUID not found, skip
                    continue;
                }
                Err(inner) => {
                    return Err(inner.into());
                }
            }

            sqlx::query!(
                r#"
                    UPDATE uuid
                        SET trashed = ?
                        WHERE id = ?
                "#,
                payload.trashed,
                id
            )
            .execute(&mut transaction)
            .await?;

            SetUuidStateEventPayload::new(payload.trashed, payload.user_id, id)
                .save(&mut transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::create_database_pool;
    use crate::event::test_helpers::fetch_age_of_newest_event;

    use super::{SetUuidStatePayload, Uuid};

    #[actix_rt::test]
    async fn set_uuid_state_no_id() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Uuid::set_uuid_state(
            SetUuidStatePayload {
                ids: vec![],
                user_id: 1,
                trashed: true,
            },
            &mut transaction,
        )
        .await
        .unwrap();
    }

    #[actix_rt::test]
    async fn set_uuid_state_single_id() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Uuid::set_uuid_state(
            SetUuidStatePayload {
                ids: vec![1855],
                user_id: 1,
                trashed: true,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that the object was trashed.
        let uuid = sqlx::query!(r#"SELECT trashed FROM uuid WHERE id = ?"#, 1855)
            .fetch_one(&mut transaction)
            .await
            .unwrap();
        assert!(uuid.trashed != 0);

        // Verify that the event was created.
        let duration = fetch_age_of_newest_event(1855, &mut transaction)
            .await
            .unwrap();
        assert!(duration < Duration::minutes(1));
    }

    #[actix_rt::test]
    async fn set_uuid_state_single_id_same_state() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Uuid::set_uuid_state(
            SetUuidStatePayload {
                ids: vec![1855],
                user_id: 1,
                trashed: false,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that the object is not trashed.
        let uuid = sqlx::query!(r#"SELECT trashed FROM uuid WHERE id = ?"#, 1855)
            .fetch_one(&mut transaction)
            .await
            .unwrap();
        assert!(uuid.trashed == 0);

        // Verify that no event was created.
        let duration = fetch_age_of_newest_event(1855, &mut transaction)
            .await
            .unwrap();
        assert!(duration > Duration::minutes(1));
    }
}
