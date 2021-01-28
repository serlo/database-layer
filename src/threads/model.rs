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
pub struct ThreadSetAchivePayload {
    ids: Vec<i32>,
    user_id: i32,
    archived: bool,
}

#[derive(Error, Debug)]
pub enum ThreadSetAchiveError {
    #[error("Thread archived state cannot be set because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Thread archived state cannot be set because of an internal error: {inner:?}.")]
    EventError { inner: EventError },
}

impl From<sqlx::Error> for ThreadSetAchiveError {
    fn from(inner: sqlx::Error) -> Self {
        ThreadSetAchiveError::DatabaseError { inner }
    }
}

impl From<EventError> for ThreadSetAchiveError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => ThreadSetAchiveError::EventError { inner },
        }
    }
}

#[derive(Serialize)]
pub struct ThreadSetAchiveResponse {
    success: bool,
}

impl Threads {
    pub async fn set_archive<'a, E>(
        payload: ThreadSetAchivePayload,
        executor: E,
    ) -> Result<ThreadSetAchiveResponse, ThreadSetAchiveError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for id in payload.ids.into_iter() {
            sqlx::query!(
                r#"
                    UPDATE comment
                        SET archived = ?
                        WHERE archived != ? AND id = ?
                "#,
                payload.archived,
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

        Ok(ThreadSetAchiveResponse { success: true })
    }
}
