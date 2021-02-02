use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;
use crate::datetime::DateTime;
use crate::event::{CreateCommentEventPayload, EventError, SetThreadStateEventPayload};
use crate::subscriptions::{Subscription, SubscriptionsError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Threads {
    pub first_comment_ids: Vec<i32>,
}

#[derive(Error, Debug)]
pub enum ThreadsError {
    #[error("Threads cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for ThreadsError {
    fn from(inner: sqlx::Error) -> Self {
        ThreadsError::DatabaseError { inner }
    }
}

impl Threads {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Self, ThreadsError> {
        Self::fetch_via_transaction(id, pool).await
    }

    pub async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Self, ThreadsError>
    where
        E: Executor<'a>,
    {
        let result = sqlx::query!(
            r#"SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC"#,
            id
        )
        .fetch_all(executor)
        .await?;

        let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

        Ok(Self { first_comment_ids })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadSetArchivePayload {
    ids: Vec<i32>,
    user_id: i32,
    archived: bool,
}

#[derive(Error, Debug)]
pub enum ThreadSetArchiveError {
    #[error("Thread archived state cannot be set because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Thread archived state cannot be set because of an internal error: {inner:?}.")]
    EventError { inner: EventError },
}

impl From<sqlx::Error> for ThreadSetArchiveError {
    fn from(inner: sqlx::Error) -> Self {
        ThreadSetArchiveError::DatabaseError { inner }
    }
}

impl From<EventError> for ThreadSetArchiveError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => ThreadSetArchiveError::EventError { inner },
        }
    }
}

impl Threads {
    pub async fn set_archive<'a, E>(
        payload: ThreadSetArchivePayload,
        executor: E,
    ) -> Result<(), ThreadSetArchiveError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for id in payload.ids.into_iter() {
            let result = sqlx::query!(
                r#"
                    SELECT archived FROM comment WHERE id = ?
                "#,
                id
            )
            .fetch_one(&mut transaction)
            .await;

            match result {
                Ok(comment) => {
                    // Comment has already the correct state, skip
                    if (comment.archived != 0) == payload.archived {
                        continue;
                    }
                }
                Err(sqlx::Error::RowNotFound) => {
                    // Comment not found, skip
                    continue;
                }
                Err(inner) => {
                    return Err(inner.into());
                }
            }

            sqlx::query!(
                r#"
                    UPDATE comment
                        SET archived = ?
                        WHERE id = ?
                "#,
                payload.archived,
                id
            )
            .execute(&mut transaction)
            .await?;

            SetThreadStateEventPayload::new(payload.archived, payload.user_id, id)
                .save(&mut transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadCommentThreadPayload {
    thread_id: i32,
    content: String,
    user_id: i32,
}

#[derive(Error, Debug)]
pub enum ThreadCommentThreadError {
    #[error("Comment could not be saved because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Comment could not be saved because thread is archived")]
    ThreadArchivedError,
    #[error("Comment could not be saved because of a database error: {inner:?}.")]
    EventError { inner: EventError },
}

impl From<sqlx::Error> for ThreadCommentThreadError {
    fn from(inner: sqlx::Error) -> Self {
        ThreadCommentThreadError::DatabaseError { inner }
    }
}

impl From<EventError> for ThreadCommentThreadError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => ThreadCommentThreadError::EventError { inner },
        }
    }
}

impl Threads {
    pub async fn comment_thread<'a, E>(
        payload: ThreadCommentThreadPayload,
        executor: E,
    ) -> Result<(), ThreadCommentThreadError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let thread = sqlx::query!(
            r#"
                SELECT instance_id, archived
                    FROM comment
                    WHERE id = ?
            "#,
            payload.thread_id
        )
        .fetch_one(&mut transaction)
        .await?;

        if thread.archived != 0 {
            return Err(ThreadCommentThreadError::ThreadArchivedError);
        }

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator)
                    VALUES (0, 'comment')
            "#
        )
        .execute(&mut transaction)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO comment (id, date, archived, title, content, uuid_id, parent_id, author_id, instance_id )
                    VALUES (LAST_INSERT_ID(), ?, 0, NULL, ?, NULL, ?, ?, ?)
            "#,
            DateTime::now(),
            payload.content,
            payload.thread_id,
            payload.user_id,
            thread.instance_id
        )
        .execute(&mut transaction)
        .await?;

        let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?;
        let comment_id = value.id as i32;

        CreateCommentEventPayload::new(
            payload.thread_id,
            comment_id,
            payload.user_id,
            thread.instance_id,
        )
        .save(&mut transaction)
        .await?;

        let subscription = Subscription {
            object_id: payload.thread_id,
            user_id: payload.user_id,
            send_email: true,
        };
        subscription
            .save(&mut transaction)
            .await
            .map_err(|error| match error {
                SubscriptionsError::DatabaseError { inner } => EventError::from(inner),
            })?;

        transaction.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::{ThreadCommentThreadPayload, ThreadSetArchivePayload, Threads};
    use crate::create_database_pool;
    use crate::event::test_helpers::fetch_age_of_newest_event;

    #[actix_rt::test]
    async fn comment_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Threads::comment_thread(
            ThreadCommentThreadPayload {
                thread_id: 17774,
                user_id: 1,
                content: "content-test".to_string(),
            },
            &mut transaction,
        )
        .await
        .unwrap();

        let comment = sqlx::query!(
            r#"
                SELECT content, author_id FROM comment WHERE parent_id = ?
            "#,
            17774
        )
        .fetch_one(&mut transaction)
        .await
        .unwrap();

        assert_eq!(comment.content, Some("content-test".to_string()));
        assert_eq!(comment.author_id, 1);
    }

    #[actix_rt::test]
    async fn comment_thread_non_existing_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let result = Threads::comment_thread(
            ThreadCommentThreadPayload {
                thread_id: 3, //does not exist
                user_id: 1,
                content: "content-test".to_string(),
            },
            &mut transaction,
        )
        .await;

        assert!(result.is_err())
    }

    #[actix_rt::test]
    async fn set_archive_no_id() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Threads::set_archive(
            ThreadSetArchivePayload {
                ids: vec![],
                user_id: 1,
                archived: true,
            },
            &mut transaction,
        )
        .await
        .unwrap();
    }

    #[actix_rt::test]
    async fn set_archive_single_id() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Threads::set_archive(
            ThreadSetArchivePayload {
                ids: vec![17666],
                user_id: 1,
                archived: true,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that the thread was archived.
        let thread = sqlx::query!(r#"SELECT archived FROM comment WHERE id = ?"#, 17666)
            .fetch_one(&mut transaction)
            .await
            .unwrap();
        assert!(thread.archived != 0);

        // Verify that the event was created.
        let duration = fetch_age_of_newest_event(17666, &mut transaction)
            .await
            .unwrap();
        assert!(duration < Duration::minutes(1));
    }

    #[actix_rt::test]
    async fn set_archive_single_id_same_state() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Threads::set_archive(
            ThreadSetArchivePayload {
                ids: vec![17666],
                user_id: 1,
                archived: false,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that the thread is not archived.
        let thread = sqlx::query!(r#"SELECT archived FROM comment WHERE id = ?"#, 17666)
            .fetch_one(&mut transaction)
            .await
            .unwrap();
        assert!(thread.archived == 0);

        // Verify that no event was created.
        let duration = fetch_age_of_newest_event(17666, &mut transaction)
            .await
            .unwrap();
        assert!(duration > Duration::minutes(1));
    }
}
