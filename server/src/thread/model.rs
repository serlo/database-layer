use super::messages::{
    create_comment_mutation, create_thread_mutation, set_thread_archived_mutation,
    update_comment_mutation,
};
use serde::Serialize;
use sqlx::MySqlPool;

use crate::database::Executor;
use crate::datetime::DateTime;
use crate::event::{
    CreateCommentEventPayload, CreateThreadEventPayload, SetThreadStateEventPayload,
};
use crate::instance::Instance;
use crate::operation;
use crate::subscription::Subscription;
use crate::uuid::{Uuid, UuidFetcher};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Threads {
    pub first_comment_ids: Vec<i32>,
}

impl Threads {
    pub async fn fetch_all_threads<'a, E>(
        first: i32,
        after: Option<String>,
        instance: Option<Instance>,
        executor: E,
    ) -> Result<Self, operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let instance_id = match instance.as_ref() {
            Some(instance) => Some(Instance::fetch_id(instance, &mut transaction).await?),
            None => None,
        };

        let after_parsed = match after.as_ref() {
            Some(date) => DateTime::parse_from_rfc3339(date)?,
            None => DateTime::now(),
        };

        // TODO: use alias for MAX(GREATEST(...)) when sqlx supports it
        let result = sqlx::query!(
            r#"
                SELECT comment.id
                FROM comment
                JOIN uuid ON uuid.id = comment.id
                JOIN comment answer ON comment.id = answer.parent_id OR
                    comment.id = answer.id
                JOIN uuid parent_uuid ON parent_uuid.id = comment.uuid_id
                WHERE
                    comment.uuid_id IS NOT NULL
                    AND uuid.trashed = 0
                    AND comment.archived = 0
                    AND (? is null OR comment.instance_id = ?)
                    AND parent_uuid.discriminator != "user"
                GROUP BY comment.id
                HAVING MAX(GREATEST(answer.date, comment.date)) < ?
                ORDER BY MAX(GREATEST(answer.date, comment.date)) DESC
                LIMIT ?
            "#,
            instance_id,
            instance_id,
            after_parsed,
            first
        )
        .fetch_all(&mut transaction)
        .await?;

        let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

        Ok(Self { first_comment_ids })
    }

    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Self, sqlx::Error> {
        Self::fetch_via_transaction(id, pool).await
    }

    pub async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Self, sqlx::Error>
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

    pub async fn set_archive<'a, E>(
        payload: &set_thread_archived_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for id in payload.ids.clone().into_iter() {
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
                .await
                .map_err(|error| operation::Error::InternalServerError {
                    error: Box::new(error),
                })?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn comment_thread<'a, E>(
        payload: &create_comment_mutation::Payload,
        executor: E,
    ) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        if payload.content.is_empty() {
            return Err(operation::Error::BadRequest {
                reason: "content is empty".to_string(),
            });
        };

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
                subscription.save(&mut transaction).await?;
            }
        }

        let comment = Uuid::fetch_via_transaction(comment_id, &mut transaction)
            .await
            .map_err(|error| operation::Error::InternalServerError {
                error: Box::new(error),
            })?;

        transaction.commit().await?;

        Ok(comment)
    }

    pub async fn start_thread<'a, E>(
        payload: &create_thread_mutation::Payload,
        executor: E,
    ) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        if payload.content.is_empty() {
            return Err(operation::Error::BadRequest {
                reason: "content is empty".to_string(),
            });
        }

        let mut transaction = executor.begin().await?;

        let instance_id = sqlx::query!(
            r#"
                SELECT i.id as instance_id
                    FROM uuid
                    JOIN (
                        SELECT id, instance_id FROM attachment_container
                        UNION ALL
                        SELECT id, instance_id FROM blog_post
                        UNION ALL
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
            payload.object_id
        )
        .fetch_one(&mut transaction)
        .await?.instance_id;

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
                INSERT INTO comment ( id , date , archived , title , content , uuid_id , parent_id , author_id , instance_id )
                    VALUES (LAST_INSERT_ID(), ?, 0, ?, ?, ?, NULL, ?, ?)
            "#,
            DateTime::now(),
            payload.title,
            payload.content,
            payload.object_id,
            payload.user_id,
            instance_id
        )
        .execute(&mut transaction)
        .await?;

        let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?;
        let thread_id = value.id as i32;

        CreateThreadEventPayload::new(thread_id, payload.object_id, payload.user_id, instance_id)
            .save(&mut transaction)
            .await
            .map_err(|error| operation::Error::InternalServerError {
                error: Box::new(error),
            })?;

        if payload.subscribe {
            let subscription = Subscription {
                object_id: thread_id,
                user_id: payload.user_id,
                send_email: payload.send_email,
            };
            subscription.save(&mut transaction).await?;
        }

        let comment = Uuid::fetch_via_transaction(thread_id, &mut transaction)
            .await
            .map_err(|error| operation::Error::InternalServerError {
                error: Box::new(error),
            })?;

        transaction.commit().await?;

        Ok(comment)
    }

    pub async fn update_comment<'a, E>(
        payload: &update_comment_mutation::Payload,
        executor: E,
    ) -> Result<operation::SuccessOutput, operation::Error>
    where
        E: Executor<'a>,
    {
        if payload.content.is_empty() {
            return Err(operation::Error::BadRequest {
                reason: "content is empty".to_string(),
            });
        }

        let mut transaction = executor.begin().await?;

        let comment = sqlx::query!(
            r#"
                SELECT content, author_id, archived, trashed
                FROM comment JOIN uuid ON uuid.id = comment.id
                WHERE comment.id = ?
            "#,
            payload.comment_id
        )
        .fetch_one(&mut transaction)
        .await?;

        if payload.user_id as i64 != comment.author_id {
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

        if payload.content != comment.content.as_deref().unwrap_or("") {
            // todo: date of edit?
            sqlx::query!(
                r#"
                    UPDATE comment SET content = ? WHERE id = ?
                "#,
                payload.content,
                payload.comment_id,
            )
            .execute(&mut transaction)
            .await?;

            // todo: trigger event?
        }

        transaction.commit().await?;

        Ok(operation::SuccessOutput { success: true })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::super::messages::{
        create_comment_mutation, create_thread_mutation, set_thread_archived_mutation,
    };
    use super::Threads;
    use crate::create_database_pool;
    use crate::event::test_helpers::fetch_age_of_newest_event;

    #[actix_rt::test]
    async fn start_thread() {
        run_test_start_thread(1565).await;
    }

    #[actix_rt::test]
    async fn start_thread_on_user() {
        run_test_start_thread(1).await;
    }

    async fn run_test_start_thread(object_id: i32) {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Threads::start_thread(
            &create_thread_mutation::Payload {
                title: "title".to_string(),
                content: "content-test".to_string(),
                object_id,
                user_id: 1,
                subscribe: true,
                send_email: false,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        let thread = sqlx::query!(
            r#"
                SELECT title, content, author_id FROM comment WHERE uuid_id = ?
                ORDER BY id DESC
            "#,
            object_id
        )
        .fetch_one(&mut transaction)
        .await
        .unwrap();

        assert_eq!(
            thread.content,
            Some("content-test".to_string()),
            "object_id: {}",
            object_id
        );
        assert_eq!(
            thread.title,
            Some("title".to_string()),
            "object_id: {}",
            object_id
        );
        assert_eq!(thread.author_id, 1, "object_id: {}", object_id);
    }

    #[actix_rt::test]
    async fn start_thread_non_existing_uuid() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let result = Threads::start_thread(
            &create_thread_mutation::Payload {
                title: "title-test".to_string(),
                content: "content-test".to_string(),
                object_id: 999999,
                user_id: 1,
                subscribe: true,
                send_email: false,
            },
            &mut transaction,
        )
        .await;

        assert!(result.is_err())
    }

    #[actix_rt::test]
    async fn comment_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Threads::comment_thread(
            &create_comment_mutation::Payload {
                thread_id: 17774,
                user_id: 1,
                content: "content-test".to_string(),
                subscribe: true,
                send_email: false,
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
            &create_comment_mutation::Payload {
                thread_id: 3, //does not exist
                user_id: 1,
                content: "content-test".to_string(),
                subscribe: true,
                send_email: false,
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
            &set_thread_archived_mutation::Payload {
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
            &set_thread_archived_mutation::Payload {
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
            &set_thread_archived_mutation::Payload {
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
