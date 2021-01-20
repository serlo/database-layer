use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::User;

#[get("/user/active-authors")]
async fn active_authors(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let result = User::find_all_active_authors(db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("Could not get active authors: {:?}", e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

#[get("/user/active-reviewers")]
async fn active_reviewers(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let result = User::find_all_active_reviewers(db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("Could not get active reviewers: {:?}", e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(active_authors);
    cfg.service(active_reviewers);
}
