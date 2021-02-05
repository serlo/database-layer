use actix_web::{get, web, Responder};
use sqlx::MySqlPool;

use super::messages::{ActiveAuthorsQuery, ActiveReviewersQuery};
use crate::message::MessageResponder;

#[get("/user/active-authors")]
async fn active_authors(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let message = ActiveAuthorsQuery {};
    message.handle(db_pool.get_ref()).await
}

#[get("/user/active-reviewers")]
async fn active_reviewers(db_pool: web::Data<MySqlPool>) -> impl Responder {
    let message = ActiveReviewersQuery {};
    message.handle(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(active_authors);
    cfg.service(active_reviewers);
}
