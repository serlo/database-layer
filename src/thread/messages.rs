use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{
    ThreadCommentThreadError, ThreadCommentThreadPayload, ThreadSetArchiveError,
    ThreadSetArchivedPayload, ThreadStartThreadError, ThreadStartThreadPayload, Threads,
    ThreadsError,
};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ThreadMessage {
    ThreadsQuery(ThreadsQuery),
    ThreadCreateThreadMutation(ThreadCreateThreadMutation),
    ThreadCreateCommentMutation(ThreadCreateCommentMutation),
    ThreadSetThreadArchivedMutation(ThreadSetThreadArchivedMutation),
}

#[async_trait]
impl MessageResponder for ThreadMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            ThreadMessage::ThreadsQuery(message) => message.handle(connection).await,
            ThreadMessage::ThreadCreateThreadMutation(message) => message.handle(connection).await,
            ThreadMessage::ThreadCreateCommentMutation(message) => message.handle(connection).await,
            ThreadMessage::ThreadSetThreadArchivedMutation(message) => {
                message.handle(connection).await
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadsQuery {
    pub id: i32,
}

#[async_trait]
impl MessageResponder for ThreadsQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let threads = match connection {
            Connection::Pool(pool) => Threads::fetch(self.id, pool).await,
            Connection::Transaction(transaction) => {
                Threads::fetch_via_transaction(self.id, transaction).await
            }
        };
        match threads {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/threads/{}: {:?}", self.id, e);
                match e {
                    ThreadsError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadCreateThreadMutation {
    pub title: String,
    pub content: String,
    pub object_id: i32,
    pub user_id: i32,
    pub subscribe: bool,
    pub send_email: bool,
}

#[async_trait]
impl MessageResponder for ThreadCreateThreadMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = ThreadStartThreadPayload {
            title: self.title.clone(),
            content: self.content.clone(),
            object_id: self.object_id,
            user_id: self.user_id,
            subscribe: self.subscribe,
            send_email: self.send_email,
        };
        let response = match connection {
            Connection::Pool(pool) => Threads::start_thread(payload, pool).await,
            Connection::Transaction(transaction) => {
                Threads::start_thread(payload, transaction).await
            }
        };
        match response {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/thread/start-thread: {:?}", e);
                match e {
                    ThreadStartThreadError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().json(&None::<String>)
                    }
                    ThreadStartThreadError::EventError { .. } => {
                        HttpResponse::InternalServerError().json(&None::<String>)
                    }
                    ThreadStartThreadError::UuidError { .. } => {
                        HttpResponse::InternalServerError().json(&None::<String>)
                    }
                }
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
                            reason: Some(format!("Cannot create comment: {}", reason).to_string()),
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
