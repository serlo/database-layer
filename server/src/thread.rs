use crate::datetime::DateTime;
use crate::event::CreateThreadEventPayload;
use crate::event::{CreateCommentEventPayload, SetThreadStateEventPayload};
use crate::operation::{self, Operation};
use crate::subscription::Subscription;
use crate::uuid::{CommentStatus, Uuid, UuidFetcher};
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::message::MessageResponder;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ThreadMessage {
    ThreadsQuery(threads_query::Payload),
    ThreadCreateThreadMutation(create_thread_mutation::Payload),
    ThreadCreateCommentMutation(create_comment_mutation::Payload),
    ThreadSetThreadArchivedMutation(set_thread_archived_mutation::Payload),
    ThreadSetThreadStatusMutation(set_thread_state_mutation::Payload),
    ThreadEditCommentMutation(edit_comment_mutation::Payload),
}

#[async_trait]
impl MessageResponder for ThreadMessage {
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            ThreadMessage::ThreadsQuery(message) => message.handle(acquire_from).await,
            ThreadMessage::ThreadCreateThreadMutation(message) => {
                message.handle(acquire_from).await
            }
            ThreadMessage::ThreadCreateCommentMutation(message) => {
                message.handle(acquire_from).await
            }
            ThreadMessage::ThreadSetThreadArchivedMutation(message) => {
                message.handle(acquire_from).await
            }
            ThreadMessage::ThreadSetThreadStatusMutation(message) => {
                message.handle(acquire_from).await
            }
            ThreadMessage::ThreadEditCommentMutation(message) => message.handle(acquire_from).await,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Threads {
    pub first_comment_ids: Vec<i32>,
}

pub mod threads_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Threads;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let mut connection = acquire_from.acquire().await?;
            let result = sqlx::query!(
                "SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC",
                self.id
            )
            .fetch_all(&mut *connection)
            .await?;

            Ok(Threads {
                first_comment_ids: result.iter().map(|child| child.id as i32).collect(),
            })
        }
    }
}

pub mod create_thread_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub title: String,
        pub content: String,
        pub object_id: i32,
        pub user_id: i32,
        pub subscribe: bool,
        pub send_email: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Uuid;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            if self.content.is_empty() {
                return Err(operation::Error::BadRequest {
                    reason: "content is empty".to_string(),
                });
            }

            let mut transaction = acquire_from.begin().await?;

            let instance_id = sqlx::query!(
                r#"
                    SELECT i.id as instance_id
                        FROM uuid
                        JOIN (
                            SELECT id, instance_id FROM comment
                            UNION ALL
                            SELECT id, instance_id FROM entity
                            UNION ALL
                            SELECT er.id, e.instance_id FROM entity_revision er JOIN entity e ON er.repository_id = e.id
                            UNION ALL
                            SELECT id, instance_id FROM page_repository
                            UNION ALL
                            SELECT pr.id, p.instance_id FROM page_revision pr JOIN page_repository p ON pr.page_repository_id = p.id
                            UNION ALL
                            SELECT ta.id, t.instance_id FROM term_taxonomy ta JOIN term t ON t.id = ta.term_id
                            UNION ALL
                            SELECT user.id, 1 FROM user) u
                        JOIN instance i ON i.id = u.instance_id
                        WHERE u.id = ?
                "#,
                self.object_id
            )
            .fetch_one(&mut *transaction)
            .await.map_err(|error| match error {
                sqlx::Error::RowNotFound => operation::Error::BadRequest{
                    reason: "UUID not found".to_string(),
                },
                error => error.into(),})?
            .instance_id;

            sqlx::query!("INSERT INTO uuid (trashed, discriminator) VALUES (0, 'comment')")
                .execute(&mut *transaction)
                .await?;

            sqlx::query!(
                r#"
                    INSERT INTO comment
                            (id, date, archived, title, content, uuid_id, parent_id,
                            author_id, instance_id)
                        VALUES (LAST_INSERT_ID(), ?, 0, ?, ?, ?, NULL, ?, ?)
                "#,
                DateTime::now(),
                self.title,
                self.content,
                self.object_id,
                self.user_id,
                instance_id
            )
            .execute(&mut *transaction)
            .await?;

            let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
                .fetch_one(&mut *transaction)
                .await?;
            let thread_id = value.id as i32;

            CreateThreadEventPayload::new(thread_id, self.object_id, self.user_id, instance_id)
                .save(&mut *transaction)
                .await
                .map_err(|error| operation::Error::InternalServerError {
                    error: Box::new(error),
                })?;

            if self.subscribe {
                let subscription = Subscription {
                    object_id: thread_id,
                    user_id: self.user_id,
                    send_email: self.send_email,
                };
                subscription.save(&mut *transaction).await?;
            }

            let comment = Uuid::fetch(thread_id, &mut *transaction)
                .await
                .map_err(|error| operation::Error::InternalServerError {
                    error: Box::new(error),
                })?;

            transaction.commit().await?;

            Ok(comment)
        }
    }
}

pub mod create_comment_mutation {
    use super::*;
    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub thread_id: i32,
        pub content: String,
        pub user_id: i32,
        pub subscribe: bool,
        pub send_email: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Uuid;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            if self.content.is_empty() {
                return Err(operation::Error::BadRequest {
                    reason: "content is empty".to_string(),
                });
            };

            let mut transaction = acquire_from.begin().await?;

            let thread = sqlx::query!(
                "SELECT instance_id, archived FROM comment WHERE id = ?",
                self.thread_id
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

            sqlx::query!("INSERT INTO uuid (trashed, discriminator) VALUES (0, 'comment')")
                .execute(&mut *transaction)
                .await?;

            sqlx::query!(
                r#"
                    INSERT INTO comment
                            (id, date, archived, title, content, uuid_id,
                            parent_id, author_id, instance_id )
                        VALUES (LAST_INSERT_ID(), ?, 0, NULL, ?, NULL, ?, ?, ?)
                "#,
                DateTime::now(),
                self.content,
                self.thread_id,
                self.user_id,
                thread.instance_id
            )
            .execute(&mut *transaction)
            .await?;

            let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
                .fetch_one(&mut *transaction)
                .await?;
            let comment_id = value.id as i32;

            CreateCommentEventPayload::new(
                self.thread_id,
                comment_id,
                self.user_id,
                thread.instance_id,
            )
            .save(&mut *transaction)
            .await
            .map_err(|error| operation::Error::InternalServerError {
                error: Box::new(error),
            })?;

            if self.subscribe {
                for object_id in [self.thread_id, comment_id].iter() {
                    let subscription = Subscription {
                        object_id: *object_id,
                        user_id: self.user_id,
                        send_email: self.send_email,
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
}

pub mod set_thread_archived_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub ids: Vec<i32>,
        pub user_id: i32,
        pub archived: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = ();

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let mut transaction = acquire_from.begin().await?;

            let number_comments = self.ids.len();
            if number_comments == 0 {
                return Ok(());
            }
            let params = format!("?{}", ", ?".repeat(number_comments - 1));
            let query_str = format!("SELECT id, archived FROM comment WHERE id IN ( {params} )");
            let mut query = sqlx::query(&query_str);
            for id in self.ids.iter() {
                query = query.bind(id);
            }
            let comments = query.fetch_all(&mut *transaction).await?;
            if comments.len() < number_comments {
                return Err(operation::Error::BadRequest {
                    reason: "not all given ids are comments".to_string(),
                });
            }

            let is_archived_after = self.archived;
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

                    SetThreadStateEventPayload::new(is_archived_after, self.user_id, id)
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
    }
}

pub mod set_thread_state_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub ids: Vec<i32>,
        pub status: CommentStatus,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = operation::SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let mut transaction = acquire_from.begin().await?;

            for id in &self.ids {
                let rows_affected = sqlx::query!(
                    r#"
                        UPDATE comment
                        SET comment_status_id = (SELECT id from comment_status where name = ?)
                        WHERE comment.id = ?
                    "#,
                    self.status.to_string(),
                    id
                )
                .execute(&mut *transaction)
                .await?
                .rows_affected();

                if rows_affected != 1 {
                    return Err(operation::Error::BadRequest {
                        reason: format!("Id {} does not belong to a comment", id),
                    });
                }
            }

            transaction.commit().await?;

            Ok(operation::SuccessOutput { success: true })
        }
    }
}

pub mod edit_comment_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: u32,
        pub comment_id: u32,
        pub content: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = operation::SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            if self.content.is_empty() {
                return Err(operation::Error::BadRequest {
                    reason: "content is empty".to_string(),
                });
            }

            let mut transaction = acquire_from.begin().await?;

            let comment = sqlx::query!(
                r#"
                SELECT content, author_id, archived, trashed
                FROM comment JOIN uuid ON uuid.id = comment.id
                WHERE comment.id = ?
            "#,
                self.comment_id
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => operation::Error::BadRequest {
                    reason: "no comment with given ID".to_string(),
                },
                error => error.into(),
            })?;

            if self.user_id as i64 != comment.author_id {
                return Err(operation::Error::BadRequest {
                    reason: "given user is not author of the comment".to_string(),
                });
            }

            if comment.archived != 0 {
                return Err(operation::Error::BadRequest {
                    reason: "archived comment cannot be edited".to_string(),
                });
            }

            if comment.trashed != 0 {
                return Err(operation::Error::BadRequest {
                    reason: "trashed comment cannot be edited".to_string(),
                });
            }

            if self.content != comment.content.as_deref().unwrap_or("") {
                sqlx::query!(
                    // todo: update edit_date (after database migration)
                    // UPDATE comment SET content = ?, edit_date = ? WHERE id = ?
                    r#"
                    UPDATE comment SET content = ? WHERE id = ?
                "#,
                    self.content,
                    // DateTime::now(),
                    self.comment_id,
                )
                .execute(&mut *transaction)
                .await?;
            }

            transaction.commit().await?;

            Ok(operation::SuccessOutput { success: true })
        }
    }
}
