use async_trait::async_trait;
use futures::join;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::format_alias;
use crate::instance::Instance;
use messages::add_revision_mutation;

use crate::event::{CreateEntityRevisionEventPayload, EventError, RevisionEventPayload};
use crate::operation;
use crate::uuid::PageRevision;
pub use messages::*;

use crate::uuid::model::page::pages_query;

mod messages;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub instance: Instance,
    pub current_revision_id: Option<i32>,
    pub revision_ids: Vec<i32>,
    pub date: DateTime,
    pub license_id: i32,
}

macro_rules! fetch_one_page {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT u.trashed, i.subdomain, p.current_revision_id, p.license_id, r.title
                    FROM page_repository p
                    JOIN uuid u ON u.id = p.id
                    JOIN instance i ON i.id = p.instance_id
                    LEFT JOIN page_revision r ON r.id = p.current_revision_id
                    WHERE p.id = ?
            "#,
            $id
        )
        .fetch_one($executor)
    };
}

macro_rules! fetch_all_revisions {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"SELECT id, date FROM page_revision WHERE page_repository_id = ?"#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! to_page {
    ($id: expr, $page: expr, $revisions: expr) => {{
        let page = $page.map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let revisions = $revisions?;

        if revisions.is_empty() {
            Err(UuidError::NotFound)
        } else {
            Ok(Uuid {
                id: $id,
                trashed: page.trashed != 0,
                // TODO:
                alias: format_alias(None, $id, page.title.as_deref()),
                concrete_uuid: ConcreteUuid::Page(Page {
                    __typename: "Page".to_string(),
                    instance: page
                        .subdomain
                        .parse()
                        .map_err(|_| UuidError::InvalidInstance)?,
                    current_revision_id: page.current_revision_id,
                    revision_ids: revisions
                        .iter()
                        .rev()
                        .map(|revision| revision.id as i32)
                        .collect(),
                    date: revisions[0].date.into(),
                    license_id: page.license_id,
                }),
            })
        }
    }};
}

#[async_trait]
impl UuidFetcher for Page {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let page = fetch_one_page!(id, pool);
        let revisions = fetch_all_revisions!(id, pool);

        let (page, revisions) = join!(page, revisions);

        to_page!(id, page, revisions)
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let page = fetch_one_page!(id, &mut transaction).await;
        let revisions = fetch_all_revisions!(id, &mut transaction).await;

        transaction.commit().await?;

        to_page!(id, page, revisions)
    }
}

impl Page {
    pub async fn add_revision<'a, E>(
        payload: &add_revision_mutation::Payload,
        executor: E,
    ) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"SELECT id FROM page_repository WHERE id = ?"#,
            payload.page_id
        )
        .fetch_optional(&mut transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: "no page found for provided pageId".to_string(),
        })?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator)
                    VALUES (0, 'pageRevision')
            "#,
        )
        .execute(&mut transaction)
        .await?;

        let page_revision_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        sqlx::query!(
            r#"
                INSERT INTO page_revision (id, author_id, page_repository_id, title, content, date)
                    VALUES (?, ?, ?, ?, ?, ?)
            "#,
            page_revision_id,
            payload.user_id,
            payload.page_id,
            payload.title,
            payload.content,
            DateTime::now(),
        )
        .execute(&mut transaction)
        .await?;

        let instance_id = sqlx::query!(
            r#"
                SELECT instance_id
                    FROM page_repository
                    WHERE id = ?
            "#,
            payload.page_id
        )
        .fetch_one(&mut transaction)
        .await?
        .instance_id as i32;

        CreateEntityRevisionEventPayload::new(
            payload.page_id,
            page_revision_id,
            payload.user_id,
            instance_id,
        )
        .save(&mut transaction)
        .await?;

        Page::checkout_revision(
            &checkout_revision_mutation::Payload {
                revision_id: page_revision_id,
                user_id: payload.user_id,
                reason: "".to_string(),
            },
            &mut transaction,
        )
        .await
        .map_err(|error| operation::Error::InternalServerError {
            error: Box::new(error),
        })?;

        let uuid = PageRevision::fetch_via_transaction(page_revision_id, &mut transaction).await?;

        transaction.commit().await?;

        Ok(uuid)
    }
}

#[derive(Error, Debug)]
pub enum PageCheckoutRevisionError {
    #[error("Revision could not be checked out because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Revision could not be checked out because of an event error: {inner:?}.")]
    EventError { inner: EventError },
    #[error("Revision could not be checked out because of an UUID error: {inner:?}.")]
    UuidError { inner: UuidError },
    #[error("Revision could not be checked out because it is already the current revision of its repository.")]
    RevisionAlreadyCheckedOut,
    #[error("Revision checkout failed because the provided UUID is not a revision: {uuid:?}.")]
    InvalidRevision { uuid: Uuid },
    #[error("Revision checkout failed because its repository is invalid: {uuid:?}.")]
    InvalidRepository { uuid: Uuid },
}

impl From<sqlx::Error> for PageCheckoutRevisionError {
    fn from(inner: sqlx::Error) -> Self {
        Self::DatabaseError { inner }
    }
}

impl From<UuidError> for PageCheckoutRevisionError {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::DatabaseError { inner } => inner.into(),
            inner => Self::UuidError { inner },
        }
    }
}

impl From<EventError> for PageCheckoutRevisionError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => Self::EventError { inner },
        }
    }
}

impl Page {
    pub async fn checkout_revision<'a, E>(
        payload: &checkout_revision_mutation::Payload,
        executor: E,
    ) -> Result<(), PageCheckoutRevisionError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let revision_id = payload.revision_id;
        let revision = PageRevision::fetch_via_transaction(revision_id, &mut transaction).await?;

        if let ConcreteUuid::PageRevision(page_revision) = revision.concrete_uuid {
            let repository_id = page_revision.repository_id;

            let repository = Page::fetch_via_transaction(repository_id, &mut transaction).await?;

            if let ConcreteUuid::Page(page) = repository.concrete_uuid {
                if page.current_revision_id == Some(revision_id) {
                    return Err(PageCheckoutRevisionError::RevisionAlreadyCheckedOut);
                }

                Uuid::set_state(revision_id, false, &mut transaction).await?;

                sqlx::query!(
                    r#"
                        UPDATE page_repository
                            SET current_revision_id = ?
                            WHERE id = ?
                    "#,
                    revision_id,
                    repository_id,
                )
                .execute(&mut transaction)
                .await?;

                RevisionEventPayload::new(
                    false,
                    payload.user_id,
                    repository_id,
                    payload.revision_id,
                    payload.reason.clone(),
                    page.instance,
                )
                .save(&mut transaction)
                .await?;

                transaction.commit().await?;

                Ok(())
            } else {
                Err(PageCheckoutRevisionError::InvalidRepository { uuid: repository })
            }
        } else {
            Err(PageCheckoutRevisionError::InvalidRevision { uuid: revision })
        }
    }
}

impl Page {
    pub async fn create<'a, E>(
        payload: &create_mutation::Payload,
        executor: E,
    ) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator)
                    VALUES (0, 'page')
            "#
        )
        .execute(&mut transaction)
        .await?;

        let page_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        let instance_id = Instance::fetch_id(&payload.instance, &mut transaction).await?;

        sqlx::query!(
            r#"
                INSERT INTO page_repository (id, instance_id, license_id, discussions_enabled)
                    VALUES (?, ?, ?, ?)
            "#,
            page_id,
            instance_id,
            payload.license_id,
            payload.discussions_enabled
        )
        .execute(&mut transaction)
        .await?;

        if let Some(forum_id) = payload.forum_id {
            sqlx::query!(
                r#"
                UPDATE page_repository
                    SET forum_id = ?
                    WHERE id = ?
            "#,
                forum_id,
                page_id,
            )
            .execute(&mut transaction)
            .await?;
        };

        Page::add_revision(
            &add_revision_mutation::Payload {
                content: payload.content.clone(),
                title: payload.title.clone(),
                page_id,
                user_id: payload.user_id,
            },
            &mut transaction,
        )
        .await?;

        let page = Page::fetch_via_transaction(page_id, &mut transaction).await?;

        transaction.commit().await?;

        Ok(page)
    }
}

#[derive(Error, Debug)]
pub enum PageRejectRevisionError {
    #[error("Revision could not be rejected because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Revision could not be rejected because of an event error: {inner:?}.")]
    EventError { inner: EventError },
    #[error("Revision could not be rejected because of an UUID error: {inner:?}.")]
    UuidError { inner: UuidError },
    #[error("Revision could not be rejected out because it already has been rejected.")]
    RevisionAlreadyRejected,
    #[error("Revision could not be rejected out because it is checked out currently.")]
    RevisionCurrentlyCheckedOut,
    #[error(
        "Revision could not be rejected because the provided UUID is not a revision: {uuid:?}."
    )]
    InvalidRevision { uuid: Uuid },
    #[error("Revision could not be rejected because its repository is invalid: {uuid:?}.")]
    InvalidRepository { uuid: Uuid },
}

impl From<sqlx::Error> for PageRejectRevisionError {
    fn from(inner: sqlx::Error) -> Self {
        Self::DatabaseError { inner }
    }
}

impl From<UuidError> for PageRejectRevisionError {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::DatabaseError { inner } => inner.into(),
            inner => Self::UuidError { inner },
        }
    }
}

impl From<EventError> for PageRejectRevisionError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => Self::EventError { inner },
        }
    }
}

impl Page {
    pub async fn reject_revision<'a, E>(
        payload: &reject_revision_mutation::Payload,
        executor: E,
    ) -> Result<(), PageRejectRevisionError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let revision_id = payload.revision_id;
        let revision = PageRevision::fetch_via_transaction(revision_id, &mut transaction).await?;

        if let ConcreteUuid::PageRevision(page_revision) = revision.concrete_uuid {
            if revision.trashed {
                return Err(PageRejectRevisionError::RevisionAlreadyRejected);
            }

            let repository_id = page_revision.repository_id;

            let repository = Page::fetch_via_transaction(repository_id, &mut transaction).await?;

            if let ConcreteUuid::Page(page) = repository.concrete_uuid {
                if page.current_revision_id == Some(revision_id) {
                    return Err(PageRejectRevisionError::RevisionCurrentlyCheckedOut);
                }

                Uuid::set_state(revision_id, true, &mut transaction).await?;

                RevisionEventPayload::new(
                    true,
                    payload.user_id,
                    page_revision.repository_id,
                    payload.revision_id,
                    payload.reason.clone(),
                    page.instance,
                )
                .save(&mut transaction)
                .await?;

                transaction.commit().await?;

                Ok(())
            } else {
                Err(PageRejectRevisionError::InvalidRepository { uuid: repository })
            }
        } else {
            Err(PageRejectRevisionError::InvalidRevision { uuid: revision })
        }
    }
}

impl Page {
    pub async fn fetch_all_pages<'a, E>(
        payload: &pages_query::Payload,
        executor: E,
    ) -> Result<Vec<i32>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        Ok(sqlx::query!(
            r#"
                SELECT page_repository.id
                FROM instance, page_repository
                JOIN page_revision ON page_repository.id = page_revision.page_repository_id
                WHERE page_repository.instance_id = instance.id
                AND (? is null or instance.subdomain = ?)
                AND page_repository.current_revision_id = page_revision.id
            "#,
            payload.instance,
            payload.instance,
        )
        .fetch_all(executor)
        .await?
        .into_iter()
        .map(|result| result.id as i32)
        .collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::messages::*;
    use super::{Page, PageCheckoutRevisionError, PageRejectRevisionError, PageRevision};
    use crate::create_database_pool;
    use crate::event::test_helpers::fetch_age_of_newest_event;
    use crate::operation;
    use crate::uuid::{ConcreteUuid, Uuid, UuidFetcher};

    #[actix_rt::test]
    async fn add_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let uuid = Page::add_revision(
            &add_revision_mutation::Payload {
                content: "test content".to_string(),
                title: "test title".to_string(),
                user_id: 1,
                page_id: 19860,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        if let ConcreteUuid::PageRevision(revision) = uuid.concrete_uuid {
            assert_eq!(revision.title, "test title".to_string());
            assert_eq!(revision.content, "test content".to_string());
            assert_eq!(revision.author_id, 1);
        } else {
            panic!("Page Revision does not fulfill assertions: {:?}", uuid)
        }
    }

    #[actix_rt::test]
    async fn add_revision_page_not_found() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let result = Page::add_revision(
            &add_revision_mutation::Payload {
                content: "test content".to_string(),
                title: "test title".to_string(),
                user_id: 1,
                page_id: 1,
            },
            &mut transaction,
        )
        .await;

        if let Err(operation::Error::BadRequest { .. }) = result {
            // This is the expected branch.
        } else {
            panic!("Expected `PageNotFound` error, got: {:?}", result)
        }
    }

    #[actix_rt::test]
    async fn checkout_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Page::checkout_revision(
            &checkout_revision_mutation::Payload {
                revision_id: 33220,
                user_id: 1,
                reason: "Revert changes".to_string(),
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that revision was checked out.
        let entity = Page::fetch_via_transaction(19767, &mut transaction)
            .await
            .unwrap();
        if let ConcreteUuid::Page(page) = entity.concrete_uuid {
            assert_eq!(page.current_revision_id, Some(33220));
        } else {
            panic!("Page does not fulfill assertions: {:?}", entity)
        }

        // Verify that the event was created.
        let duration = fetch_age_of_newest_event(33220, &mut transaction)
            .await
            .unwrap();
        assert!(duration < Duration::minutes(1));
    }

    #[actix_rt::test]
    async fn checkout_revision_sets_trashed_flag_to_false() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let revision_id: i32 = 33220;
        let entity_id: i32 = 19767;

        Uuid::set_state(revision_id, true, &mut transaction)
            .await
            .unwrap();

        let entity = Page::fetch_via_transaction(entity_id, &mut transaction)
            .await
            .unwrap();
        assert!(!entity.trashed);
    }

    #[actix_rt::test]
    async fn checkout_checked_out_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let result = Page::checkout_revision(
            &checkout_revision_mutation::Payload {
                revision_id: 35476,
                user_id: 1,
                reason: "Revert changes".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(PageCheckoutRevisionError::RevisionAlreadyCheckedOut) = result {
            // This is the expected branch.
        } else {
            panic!(
                "Expected `RevisionAlreadyCheckedOut` error, got: {:?}",
                result
            )
        }
    }

    #[actix_rt::test]
    async fn reject_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Page::reject_revision(
            &reject_revision_mutation::Payload {
                revision_id: 33220,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that revision was trashed.
        let revision = PageRevision::fetch_via_transaction(33220, &mut transaction)
            .await
            .unwrap();
        assert!(revision.trashed);

        // Verify that the event was created.
        let duration = fetch_age_of_newest_event(33220, &mut transaction)
            .await
            .unwrap();
        assert!(duration < Duration::minutes(1));
    }

    #[actix_rt::test]
    async fn reject_rejected_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Uuid::set_state(33220, true, &mut transaction)
            .await
            .unwrap();

        let result = Page::reject_revision(
            &reject_revision_mutation::Payload {
                revision_id: 33220,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(PageRejectRevisionError::RevisionAlreadyRejected) = result {
            // This is the expected branch.
        } else {
            panic!(
                "Expected `RevisionAlreadyRejected` error, got: {:?}",
                result
            )
        }
    }

    #[actix_rt::test]
    async fn reject_checked_out_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let result = Page::reject_revision(
            &reject_revision_mutation::Payload {
                revision_id: 35476,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(PageRejectRevisionError::RevisionCurrentlyCheckedOut) = result {
            // This is the expected branch.
        } else {
            panic!(
                "Expected `RevisionCurrentlyCheckedOut` error, got: {:?}",
                result
            )
        }
    }
}
