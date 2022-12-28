use crate::operation;
use crate::uuid::messages::uuid_set_state_mutation;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use super::discriminator::Discriminator;
use super::{
    attachment::Attachment, blog_post::BlogPost, comment::Comment, entity::Entity,
    entity_revision::EntityRevision, page::Page, page_revision::PageRevision,
    taxonomy_term::TaxonomyTerm, user::User,
};
use crate::database::Executor;
use crate::event::SetUuidStateEventPayload;
use crate::instance::Instance;

#[derive(Debug, Serialize)]
pub struct Uuid {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    #[serde(flatten)]
    pub concrete_uuid: ConcreteUuid,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ConcreteUuid {
    Attachment,
    BlogPost,
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

impl From<UuidError> for operation::Error {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::UnsupportedDiscriminator { .. }
            | UuidError::UnsupportedEntityType { .. }
            | UuidError::UnsupportedEntityRevisionType { .. }
            | UuidError::EntityMissingRequiredParent
            | UuidError::NotFound => operation::Error::NotFoundError,
            UuidError::DatabaseError { .. } | UuidError::InvalidInstance => {
                operation::Error::InternalServerError {
                    error: Box::new(error),
                }
            }
        }
    }
}

#[async_trait]
pub trait UuidFetcher {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError>
    where
        Self: Sized;
    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
        Self: Sized;
}

#[async_trait]
pub trait AssertExists: UuidFetcher {
    async fn assert_exists<'a, E>(id: i32, executor: E) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
        Self: Sized,
    {
        if let Err(UuidError::NotFound) = Self::fetch_via_transaction(id, executor).await {
            return Err(operation::Error::BadRequest {
                reason: format!(
                    "Id {} does not exist or does not correspond to the type",
                    id
                ),
            });
        }
        Ok(())
    }
}

macro_rules! fetch_one_uuid {
    ($id: expr, $executor: expr) => {
        sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, $id)
            .fetch_one($executor)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => UuidError::NotFound,
                error => error.into(),
            })
    };
}

macro_rules! get_discriminator {
    ($uuid: expr) => {
        $uuid.discriminator.parse::<Discriminator>().map_err(|_| {
            UuidError::UnsupportedDiscriminator {
                discriminator: $uuid.discriminator,
            }
        })
    };
}

#[async_trait]
impl UuidFetcher for Uuid {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Self, UuidError> {
        let uuid = fetch_one_uuid!(id, pool)?;
        let discriminator = get_discriminator!(uuid)?;
        let uuid = match discriminator {
            Discriminator::Attachment => Attachment::fetch(id, pool).await?,
            Discriminator::BlogPost => BlogPost::fetch(id, pool).await?,
            Discriminator::Comment => Comment::fetch(id, pool).await?,
            Discriminator::Entity => Entity::fetch(id, pool).await?,
            Discriminator::EntityRevision => EntityRevision::fetch(id, pool).await?,
            Discriminator::Page => Page::fetch(id, pool).await?,
            Discriminator::PageRevision => PageRevision::fetch(id, pool).await?,
            Discriminator::TaxonomyTerm => TaxonomyTerm::fetch(id, pool).await?,
            Discriminator::User => User::fetch(id, pool).await?,
        };
        Ok(uuid)
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Self, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let uuid = fetch_one_uuid!(id, &mut transaction)?;
        let discriminator = get_discriminator!(uuid)?;
        let uuid = match discriminator {
            Discriminator::Attachment => {
                Attachment::fetch_via_transaction(id, &mut transaction).await?
            }
            Discriminator::BlogPost => {
                BlogPost::fetch_via_transaction(id, &mut transaction).await?
            }
            Discriminator::Comment => Comment::fetch_via_transaction(id, &mut transaction).await?,
            Discriminator::Entity => Entity::fetch_via_transaction(id, &mut transaction).await?,
            Discriminator::EntityRevision => {
                EntityRevision::fetch_via_transaction(id, &mut transaction).await?
            }
            Discriminator::Page => Page::fetch_via_transaction(id, &mut transaction).await?,
            Discriminator::PageRevision => {
                PageRevision::fetch_via_transaction(id, &mut transaction).await?
            }
            Discriminator::TaxonomyTerm => {
                TaxonomyTerm::fetch_via_transaction(id, &mut transaction).await?
            }
            Discriminator::User => User::fetch_via_transaction(id, &mut transaction).await?,
        };
        transaction.commit().await?;
        Ok(uuid)
    }
}

impl Uuid {
    pub async fn fetch_context(id: i32, pool: &MySqlPool) -> Result<Option<String>, UuidError> {
        let uuid = fetch_one_uuid!(id, pool)?;
        let discriminator = get_discriminator!(uuid)?;
        let context = match discriminator {
            Discriminator::Attachment => Attachment::get_context(),
            Discriminator::BlogPost => BlogPost::get_context(),
            // This is done intentionally to avoid a recursive `async fn` and because this is not needed.
            Discriminator::Comment => None,
            Discriminator::Entity => Entity::fetch_canonical_subject(id, pool)
                .await?
                .map(|subject| subject.name),
            Discriminator::EntityRevision => EntityRevision::fetch_canonical_subject(id, pool)
                .await?
                .map(|subject| subject.name),
            Discriminator::Page => None,         // TODO:
            Discriminator::PageRevision => None, // TODO:
            Discriminator::TaxonomyTerm => TaxonomyTerm::fetch_canonical_subject(id, pool)
                .await?
                .map(|subject| subject.name),
            Discriminator::User => User::get_context(),
        };
        Ok(context)
    }

    pub async fn fetch_context_via_transaction<'a, E>(
        id: i32,
        executor: E,
    ) -> Result<Option<String>, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let uuid = fetch_one_uuid!(id, &mut transaction)?;
        let discriminator = get_discriminator!(uuid)?;
        let context = match discriminator {
            Discriminator::Attachment => Attachment::get_context(),
            Discriminator::BlogPost => BlogPost::get_context(),
            // This is done intentionally to avoid a recursive `async fn` and because this is not needed.
            Discriminator::Comment => None,
            Discriminator::Entity => {
                Entity::fetch_canonical_subject_via_transaction(id, &mut transaction)
                    .await?
                    .map(|subject| subject.name)
            }
            Discriminator::EntityRevision => {
                EntityRevision::fetch_canonical_subject_via_transaction(id, &mut transaction)
                    .await?
                    .map(|subject| subject.name)
            }
            Discriminator::Page => None,         // TODO:
            Discriminator::PageRevision => None, // TODO:
            Discriminator::TaxonomyTerm => {
                TaxonomyTerm::fetch_canonical_subject(id, &mut transaction)
                    .await?
                    .map(|subject| subject.name)
            }
            Discriminator::User => User::get_context(),
        };
        transaction.commit().await?;
        Ok(context)
    }

    pub fn get_alias(&self) -> String {
        self.alias.clone()
    }
}

impl Uuid {
    pub async fn set_uuid_state<'a, E>(
        payload: &uuid_set_state_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for id in &payload.ids {
            let result = sqlx::query!(
                r#"
                    SELECT u.trashed, i.subdomain, u.discriminator
                        FROM uuid u
                        JOIN (
                        SELECT id, instance_id FROM attachment_container
                        UNION ALL
                        SELECT id, instance_id FROM blog_post
                        UNION ALL
                        SELECT id, instance_id FROM comment
                        UNION ALL
                        SELECT id, instance_id FROM entity
                        UNION ALL
                        SELECT id, instance_id FROM page_repository
                        UNION ALL
                        SELECT pr.id, p.instance_id FROM page_revision pr JOIN page_repository p ON pr.page_repository_id = p.id
                        UNION ALL
                        SELECT term_taxonomy.id, instance_id FROM term_taxonomy JOIN term ON term.id = term_taxonomy.term_id
                        ) c ON c.id = u.id
                        JOIN instance i ON i.id = c.instance_id
                        WHERE u.id = ?
                "#,
                id
            )
            .fetch_one(&mut transaction)
            .await;
            match result {
                Ok(uuid) => {
                    // Actually the query already exludes entity revisions and users.
                    // But we can leave it as kind of reminder in case the query is wrongly refactored.
                    if uuid.discriminator == "entityRevision" || uuid.discriminator == "user" {
                        return Err(operation::Error::BadRequest {
                            reason: format!(
                                "uuid {} with type \"{}\" cannot be deleted via a setState mutation",
                                id,
                                uuid.discriminator
                            ),
                        });
                    }

                    // UUID has already the correct state, skip
                    if (uuid.trashed != 0) == payload.trashed {
                        continue;
                    }
                    let instance: Instance = uuid.subdomain.parse().map_err(|error| {
                        operation::Error::InternalServerError {
                            error: Box::new(error),
                        }
                    })?;

                    Uuid::set_state(*id, payload.trashed, &mut transaction).await?;

                    SetUuidStateEventPayload::new(payload.trashed, payload.user_id, *id, instance)
                        .save(&mut transaction)
                        .await?;
                }
                Err(sqlx::Error::RowNotFound) => {
                    return Err(operation::Error::BadRequest {
                        reason: "Uuid does not exist or cannot be trashed".to_string(),
                    })
                }
                Err(inner) => {
                    return Err(inner.into());
                }
            };
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn set_state<'a, E>(
        id: i32,
        trashed: bool,
        executor: E,
    ) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error>
    where
        E: Executor<'a>,
    {
        sqlx::query!("UPDATE uuid SET trashed = ? WHERE id = ?", trashed, id)
            .execute(executor)
            .await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::create_database_pool;
    use crate::event::test_helpers::fetch_age_of_newest_event;

    use super::{uuid_set_state_mutation, Uuid};

    #[actix_rt::test]
    async fn set_uuid_state_no_id() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Uuid::set_uuid_state(
            &uuid_set_state_mutation::Payload {
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
            &uuid_set_state_mutation::Payload {
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
            &uuid_set_state_mutation::Payload {
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
