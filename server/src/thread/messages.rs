use crate::instance::Instance;
use crate::operation::{self, Operation};
use crate::uuid::Uuid;
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::Threads;
use crate::message::MessageResponder;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ThreadMessage {
    AllThreadsQuery(all_threads_query::Payload),
    ThreadsQuery(threads_query::Payload),
    ThreadCreateThreadMutation(create_thread_mutation::Payload),
    ThreadCreateCommentMutation(create_comment_mutation::Payload),
    ThreadSetThreadArchivedMutation(set_thread_archived_mutation::Payload),
    ThreadEditCommentMutation(edit_comment_mutation::Payload),
}

#[async_trait]
impl MessageResponder for ThreadMessage {
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            ThreadMessage::AllThreadsQuery(message) => message.handle(acquire_from).await,
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
            ThreadMessage::ThreadEditCommentMutation(message) => message.handle(acquire_from).await,
        }
    }
}

pub mod all_threads_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub first: i32,
        pub after: Option<String>,
        pub instance: Option<Instance>,
        pub subject_id: Option<i32>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Threads;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let mut transaction = acquire_from.begin().await?;

            let instance_id = match self.instance.as_ref() {
                Some(instance) => Some(Instance::fetch_id(&instance, &mut *transaction).await?),
                None => None,
            };

            let after_parsed = match self.after.as_ref() {
                Some(date) => DateTime::parse_from_rfc3339(date.as_str())?,
                None => DateTime::now(),
            };

            // TODO: use alias for MAX(GREATEST(...)) when sqlx supports it
            let result = sqlx::query!(
                r#"
                WITH RECURSIVE descendants AS (
                    SELECT id, parent_id
                    FROM term_taxonomy
                    WHERE (? is null OR id = ?)

                    UNION

                    SELECT tt.id, tt.parent_id
                    FROM term_taxonomy tt
                    JOIN descendants d ON tt.parent_id = d.id
                ), subject_entities AS (
                SELECT id as entity_id FROM descendants

                UNION

                SELECT tte.entity_id
                FROM descendants
                JOIN term_taxonomy_entity tte ON descendants.id = tte.term_taxonomy_id

                UNION

                SELECT entity_link.child_id
                FROM descendants
                JOIN term_taxonomy_entity tte ON descendants.id = tte.term_taxonomy_id
                JOIN entity_link ON entity_link.parent_id = tte.entity_id

                UNION

                SELECT entity_link.child_id
                FROM descendants
                JOIN term_taxonomy_entity tte ON descendants.id = tte.term_taxonomy_id
                JOIN entity_link parent_link ON parent_link.parent_id = tte.entity_id
                JOIN entity_link ON entity_link.parent_id = parent_link.child_id
                )
                SELECT comment.id
                FROM comment
                JOIN uuid ON uuid.id = comment.id
                JOIN comment answer ON comment.id = answer.parent_id OR
                    comment.id = answer.id
                JOIN uuid parent_uuid ON parent_uuid.id = comment.uuid_id
                JOIN subject_entities ON subject_entities.entity_id = comment.uuid_id
                WHERE
                    comment.uuid_id IS NOT NULL
                    AND uuid.trashed = 0
                    AND comment.archived = 0
                    AND (? is null OR comment.instance_id = ?)
                    AND parent_uuid.discriminator != "user"
                GROUP BY comment.id
                HAVING MAX(GREATEST(answer.date, comment.date)) < ?
                ORDER BY MAX(GREATEST(answer.date, comment.date)) DESC
                LIMIT ?;
            "#,
                self.subject_id,
                self.subject_id,
                instance_id,
                instance_id,
                after_parsed,
                self.first
            )
            .fetch_all(&mut *transaction)
            .await?;

            let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

            Ok(Threads { first_comment_ids })
        }
    }
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
            Ok(Threads::fetch(self.id, acquire_from).await?)
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
            Ok(Threads::start_thread(self, acquire_from).await?)
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
            Ok(Threads::comment_thread(self, acquire_from).await?)
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
            Threads::set_archive(self, acquire_from).await?;
            Ok(())
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
            Ok(Threads::edit_comment(self, acquire_from).await?)
        }
    }
}
