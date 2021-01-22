use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{User, UserError};

#[get("/user/active-authors")]
async fn active_authors(db_pool: web::Data<MySqlPool>) -> impl Responder {
    match User::fetch_active_authors(db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/user/active-authors: {:?}", e);
            match e {
                UserError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

#[get("/user/active-reviewers")]
async fn active_reviewers(db_pool: web::Data<MySqlPool>) -> impl Responder {
    match User::fetch_active_reviewers(db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/user/active-reviewers: {:?}", e);
            match e {
                UserError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(active_authors);
    cfg.service(active_reviewers);
}
