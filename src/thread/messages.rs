use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{Threads, ThreadsError};
use crate::message::MessageResponder;
use crate::thread::model::{
    ThreadCommentThreadError, ThreadCommentThreadPayload, ThreadStartThreadError,
    ThreadStartThreadPayload,
};
use crate::thread::{ThreadSetArchiveError, ThreadSetArchivedPayload};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ThreadMessage {
    ThreadsQuery(ThreadsQuery),
    ThreadCreateThreadMutation(ThreadCreateThreadMutation),
    ThreadCreateCommentMutation(ThreadCreateCommentMutation),
}

#[async_trait]
impl MessageResponder for ThreadMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            ThreadMessage::ThreadsQuery(message) => message.handle(pool).await,
            ThreadMessage::ThreadCreateThreadMutation(message) => message.handle(pool).await,
            ThreadMessage::ThreadCreateCommentMutation(message) => message.handle(pool).await,
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
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match Threads::fetch(self.id, pool).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
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
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        let payload = ThreadStartThreadPayload {
            title: self.title.clone(),
            content: self.content.clone(),
            object_id: self.object_id,
            user_id: self.user_id,
            subscribe: self.subscribe,
            send_email: self.send_email,
        };
        match Threads::start_thread(payload, pool).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                println!("/thread/start-thread: {:?}", e);
                match e {
                    ThreadStartThreadError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().json(None::<String>)
                    }
                    ThreadStartThreadError::EventError { .. } => {
                        HttpResponse::InternalServerError().json(None::<String>)
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

#[async_trait]
impl MessageResponder for ThreadCreateCommentMutation {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        let payload = ThreadCommentThreadPayload {
            thread_id: self.thread_id,
            content: self.content.clone(),
            user_id: self.user_id,
            subscribe: self.subscribe,
            send_email: self.send_email,
        };
        match Threads::comment_thread(payload, pool).await {
            Ok(_) => HttpResponse::Ok().finish(),
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
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        let payload = ThreadSetArchivedPayload {
            ids: self.ids.clone(),
            user_id: self.user_id,
            archived: self.archived,
        };
        match Threads::set_archive(payload, pool).await {
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
