use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{
    ThreadCommentThreadError, ThreadCommentThreadPayload, ThreadSetArchiveError,
    ThreadSetArchivePayload, Threads, ThreadsError,
};

#[get("/threads/{id}")]
async fn threads(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    match Threads::fetch(id, db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/threads/{}: {:?}", id, e);
            match e {
                ThreadsError::DatabaseError { .. } => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

#[post("/thread/set-archive")]
async fn set_archive(
    payload: web::Json<ThreadSetArchivePayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    match Threads::set_archive(payload.into_inner(), db_pool.get_ref()).await {
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

#[post("/thread/comment-thread")]
async fn comment_thread(
    payload: web::Json<ThreadCommentThreadPayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    match Threads::comment_thread(payload.into_inner(), db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("/thread/comment-thread: {:?}", e);
            match e {
                ThreadCommentThreadError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                ThreadCommentThreadError::ThreadArchivedError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                ThreadCommentThreadError::EventError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(threads);
    cfg.service(set_archive);
    cfg.service(comment_thread);
}
