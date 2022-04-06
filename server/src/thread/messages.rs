use crate::operation::{self, Operation};
use crate::uuid::Uuid;
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{
    ThreadCommentThreadError, ThreadCommentThreadPayload, ThreadSetArchiveError,
    ThreadSetArchivedPayload, ThreadStartThreadError, ThreadStartThreadPayload, Threads,
};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ThreadMessage {
    ThreadsQuery(threads_query::Payload),
    ThreadCreateThreadMutation(create_thread_mutation::Payload),
    ThreadCreateCommentMutation(ThreadCreateCommentMutation),
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
            ThreadMessage::ThreadCreateCommentMutation(message) => message.handle(connection).await,
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
            let payload = ThreadStartThreadPayload {
                title: self.title.clone(),
                content: self.content.clone(),
                object_id: self.object_id,
                user_id: self.user_id,
                subscribe: self.subscribe,
                send_email: self.send_email,
            };
            Ok(match connection {
                Connection::Pool(pool) => Threads::start_thread(payload, pool).await?,
                Connection::Transaction(transaction) => {
                    Threads::start_thread(payload, transaction).await?
                }
            })
        }
    }

    impl From<ThreadStartThreadError> for operation::Error {
        fn from(e: ThreadStartThreadError) -> Self {
            match e {
                ThreadStartThreadError::DatabaseError { inner } => {
                    operation::Error::InternalServerError {
                        error: Box::new(inner),
                    }
                }
                ThreadStartThreadError::EventError { inner } => {
                    operation::Error::InternalServerError {
                        error: Box::new(inner),
                    }
                }
                ThreadStartThreadError::UuidError { inner } => {
                    operation::Error::InternalServerError {
                        error: Box::new(inner),
                    }
                }
                ThreadStartThreadError::BadUserInput { reason } => operation::Error::BadRequest {
                    reason: format!("Cannot create thread: {}", reason).to_string(),
                },
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadCreateCommentMutation {
    pub thread_id: i32,
    pub content: String,
    pub user_id: i32,
    pub subscribe: bool,
    pub send_email: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadCreateCommentMutationResponse {
    pub success: bool,
    pub reason: Option<String>,
}

#[async_trait]
impl MessageResponder for ThreadCreateCommentMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = ThreadCommentThreadPayload {
            thread_id: self.thread_id,
            content: self.content.clone(),
            user_id: self.user_id,
            subscribe: self.subscribe,
            send_email: self.send_email,
        };
        let response = match connection {
            Connection::Pool(pool) => Threads::comment_thread(payload, pool).await,
            Connection::Transaction(transaction) => {
                Threads::comment_thread(payload, transaction).await
            }
        };
        match response {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/thread/comment-thread: {:?}", e);
                match e {
                    ThreadCommentThreadError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    ThreadCommentThreadError::ThreadArchivedError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    ThreadCommentThreadError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    ThreadCommentThreadError::UuidError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    ThreadCommentThreadError::BadUserInput { reason } => HttpResponse::BadRequest()
                        .content_type("application/json; charset=utf-8")
                        .json(ThreadCreateCommentMutationResponse {
                            success: false,
                            reason: Some(format!("Cannot create comment: {}", reason)),
                        }),
                }
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
