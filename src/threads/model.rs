use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;
use crate::event::{EventError, SetThreadStateEventPayload};

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
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Threads, ThreadsError> {
        let result = sqlx::query!(
            r#"SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC"#,
            id
        )
        .fetch_all(pool)
        .await?;

        let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

        Ok(Threads { first_comment_ids })
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

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::{ThreadSetArchivePayload, Threads};
    use crate::create_database_pool;
    use crate::event::test_helpers::fetch_age_of_newest_event;

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
