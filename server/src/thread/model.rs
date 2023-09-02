use super::messages::{create_comment_mutation, set_thread_archived_mutation};
use serde::Serialize;
use sqlx::Row;

use crate::datetime::DateTime;
use crate::event::{CreateCommentEventPayload, SetThreadStateEventPayload};
use crate::operation;
use crate::subscription::Subscription;
use crate::uuid::{Uuid, UuidFetcher};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Threads {
    pub first_comment_ids: Vec<i32>,
}

impl Threads {
    pub async fn fetch<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        id: i32,
        acquire_from: A,
    ) -> Result<Self, sqlx::Error> {
        let mut connection = acquire_from.acquire().await?;
        let result = sqlx::query!(
            r#"SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC"#,
            id
        )
        .fetch_all(&mut *connection)
        .await?;

        let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

        Ok(Self { first_comment_ids })
    }

    pub async fn set_archive<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &set_thread_archived_mutation::Payload,
        acquire_from: A,
    ) -> Result<(), operation::Error> {
        let mut transaction = acquire_from.begin().await?;

        let number_comments = payload.ids.len();
        if number_comments == 0 {
            return Ok(());
        }
        let params = format!("?{}", ", ?".repeat(number_comments - 1));
        let query_str = format!("SELECT id, archived FROM comment WHERE id IN ( {params} )");
        let mut query = sqlx::query(&query_str);
        for id in payload.ids.iter() {
            query = query.bind(id);
        }
        let comments = query.fetch_all(&mut *transaction).await?;
        if comments.len() < number_comments {
            return Err(operation::Error::BadRequest {
                reason: "not all given ids are comments".to_string(),
            });
        }

        let is_archived_after = payload.archived;
        for comment in comments {
            let id: i32 = comment.get("id");
            let is_archived_before: bool = comment.get("archived");
            if is_archived_after != is_archived_before {
                sqlx::query!(
                    r#"
                        UPDATE comment
                            SET archived = ?
                            WHERE id = ?
                    "#,
                    is_archived_after,
                    id
                )
                .execute(&mut *transaction)
                .await?;

                SetThreadStateEventPayload::new(is_archived_after, payload.user_id, id)
                    .save(&mut *transaction)
                    .await
                    .map_err(|error| operation::Error::InternalServerError {
                        error: Box::new(error),
                    })?;
            }
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn comment_thread<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &create_comment_mutation::Payload,
        acquire_from: A,
    ) -> Result<Uuid, operation::Error> {
        if payload.content.is_empty() {
            return Err(operation::Error::BadRequest {
                reason: "content is empty".to_string(),
            });
        };

        let mut transaction = acquire_from.begin().await?;

        let thread = sqlx::query!(
            r#"
                SELECT instance_id, archived
                    FROM comment
                    WHERE id = ?
            "#,
            payload.thread_id
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => operation::Error::BadRequest {
                reason: "thread does not exist".to_string(),
            },
            error => error.into(),
        })?;

        if thread.archived != 0 {
            // TODO: test is missing
            return Err(operation::Error::BadRequest {
                reason: "thread is already archived".to_string(),
            });
        }

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator)
                    VALUES (0, 'comment')
            "#
        )
        .execute(&mut *transaction)
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
        .execute(&mut *transaction)
        .await?;

        let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut *transaction)
            .await?;
        let comment_id = value.id as i32;

        CreateCommentEventPayload::new(
            payload.thread_id,
            comment_id,
            payload.user_id,
            thread.instance_id,
        )
        .save(&mut *transaction)
        .await
        .map_err(|error| operation::Error::InternalServerError {
            error: Box::new(error),
        })?;

        if payload.subscribe {
            for object_id in [payload.thread_id, comment_id].iter() {
                let subscription = Subscription {
                    object_id: *object_id,
                    user_id: payload.user_id,
                    send_email: payload.send_email,
                };
                subscription.save(&mut *transaction).await?;
            }
        }

        let comment = Uuid::fetch(comment_id, &mut *transaction)
            .await
            .map_err(|error| operation::Error::InternalServerError {
                error: Box::new(error),
            })?;

        transaction.commit().await?;

        Ok(comment)
    }
}
