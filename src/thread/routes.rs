use actix_web::{get, post, web, Responder};
use sqlx::MySqlPool;

use super::messages::{
    ThreadCreateCommentMutation, ThreadCreateThreadMutation, ThreadSetThreadArchivedMutation,
    ThreadsQuery,
};
use super::model::{
    ThreadCommentThreadPayload, ThreadSetArchivedPayload, ThreadStartThreadPayload,
};
use crate::message::MessageResponder;

#[get("/threads/{id}")]
async fn threads(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let message = ThreadsQuery { id };
    message.handle(db_pool.get_ref()).await
}

#[post("/thread/set-archive")]
async fn set_archive(
    payload: web::Json<ThreadSetArchivedPayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let payload = payload.into_inner();
    let message = ThreadSetThreadArchivedMutation {
        ids: payload.ids.clone(),
        user_id: payload.user_id,
        archived: payload.archived,
    };
    message.handle(db_pool.get_ref()).await
}

#[post("/thread/start-thread")]
async fn start_thread(
    payload: web::Json<ThreadStartThreadPayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let payload = payload.into_inner();
    let message = ThreadCreateThreadMutation {
        title: payload.title.clone(),
        content: payload.content.clone(),
        object_id: payload.object_id,
        user_id: payload.user_id,
        subscribe: payload.subscribe,
        send_email: payload.send_email,
    };
    message.handle(db_pool.get_ref()).await
}

#[post("/thread/comment-thread")]
async fn comment_thread(
    payload: web::Json<ThreadCommentThreadPayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let payload = payload.into_inner();
    let message = ThreadCreateCommentMutation {
        thread_id: payload.thread_id,
        content: payload.content.clone(),
        user_id: payload.user_id,
        subscribe: payload.subscribe,
        send_email: payload.send_email,
    };
    message.handle(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(threads);
    cfg.service(set_archive);
    cfg.service(comment_thread);
    cfg.service(start_thread);
}
