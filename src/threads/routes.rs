use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{ThreadSetAchiveError, ThreadSetAchivePayload, Threads, ThreadsError};

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
                ThreadsError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

#[post("/thread/set-archive")]
async fn set_archive(
    payload: web::Json<ThreadSetAchivePayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    match Threads::set_archive(payload.into_inner(), db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/set-uuid-state/: {:?}", e);
            match e {
                ThreadSetAchiveError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                ThreadSetAchiveError::EventError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(threads);
    cfg.service(set_archive);
}
