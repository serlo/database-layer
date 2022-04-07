use crate::operation::{self, Operation};
use crate::uuid::Uuid;
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::Threads;
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ThreadMessage {
    ThreadsQuery(threads_query::Payload),
    ThreadCreateThreadMutation(create_thread_mutation::Payload),
    ThreadCreateCommentMutation(create_comment_mutation::Payload),
    ThreadSetThreadArchivedMutation(set_thread_archived_mutation::Payload),
}

#[async_trait]
impl MessageResponder for ThreadMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            ThreadMessage::ThreadsQuery(message) => {
                message.handle("ThreadsQuery", connection).await
            }
            ThreadMessage::ThreadCreateThreadMutation(message) => {
                message
                    .handle("ThreadCreateCommentMutation", connection)
                    .await
            }
            ThreadMessage::ThreadCreateCommentMutation(message) => {
                message
                    .handle("ThreadCreateCommentMutation", connection)
                    .await
            }
            ThreadMessage::ThreadSetThreadArchivedMutation(message) => {
                message
                    .handle("ThreadSetThreadArchivedMutation", connection)
                    .await
            }
        }
    }
}

pub mod threads_query {
    use super::*;
    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Threads;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => Threads::fetch(self.id, pool).await?,
                Connection::Transaction(transaction) => {
                    Threads::fetch_via_transaction(self.id, transaction).await?
                }
            })
        }
    }
}

pub mod create_thread_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => Threads::start_thread(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Threads::start_thread(self, transaction).await?
                }
            })
        }
    }
}

pub mod create_comment_mutation {
    use super::*;
    #[derive(Deserialize, Serialize)]
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => Threads::comment_thread(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Threads::comment_thread(self, transaction).await?
                }
            })
        }
    }
}

pub mod set_thread_archived_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub ids: Vec<i32>,
        pub user_id: i32,
        pub archived: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = ();

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => Threads::set_archive(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Threads::set_archive(self, transaction).await?
                }
            }
            Ok(())
        }
    }
}
