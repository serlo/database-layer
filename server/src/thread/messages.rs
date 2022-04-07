use crate::operation::{self, Operation};
use crate::uuid::Uuid;
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{
    ThreadCommentThreadError, ThreadSetArchiveError, ThreadSetArchivedPayload, Threads,
};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ThreadMessage {
    ThreadsQuery(threads_query::Payload),
    ThreadCreateThreadMutation(create_thread_mutation::Payload),
    ThreadCreateCommentMutation(create_comment_mutation::Payload),
    ThreadSetThreadArchivedMutation(ThreadSetThreadArchivedMutation),
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
                message.handle(connection).await
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

    impl From<ThreadCommentThreadError> for operation::Error {
        fn from(e: ThreadCommentThreadError) -> Self {
            match e {
                ThreadCommentThreadError::BadUserInput { reason } => {
                    operation::Error::BadRequest { reason }
                }
                _ => operation::Error::InternalServerError { error: Box::new(e) },
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadSetThreadArchivedMutation {
    pub ids: Vec<i32>,
    pub user_id: i32,
    pub archived: bool,
}

#[async_trait]
impl MessageResponder for ThreadSetThreadArchivedMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = ThreadSetArchivedPayload {
            ids: self.ids.clone(),
            user_id: self.user_id,
            archived: self.archived,
        };
        let response = match connection {
            Connection::Pool(pool) => Threads::set_archive(payload, pool).await,
            Connection::Transaction(transaction) => {
                Threads::set_archive(payload, transaction).await
            }
        };
        match response {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                println!("/thread/set-archive: {:?}", e);
                match e {
                    ThreadSetArchiveError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    ThreadSetArchiveError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}
