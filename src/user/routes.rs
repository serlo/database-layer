use crate::user::model::User;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/user/active-authors")]
async fn get_active_authors(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let result = User::get_active_author_ids(db_pool.get_ref()).await;
    match result {
        Ok(user_array) => HttpResponse::Ok().json(user_array),
        Err(e) => {
            println!("Could not get active authors: {:?}", e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

#[get("/user/active-reviewers")]
async fn get_active_reviewers(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let result = User::get_active_reviewer_ids(db_pool.get_ref()).await;
    match result {
        Ok(user_array) => HttpResponse::Ok().json(user_array),
        Err(e) => {
            println!("Could not get active reviewers: {:?}", e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_active_authors);
}
