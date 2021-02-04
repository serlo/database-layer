use actix_web::{get, web, Responder};
use sqlx::MySqlPool;

use super::messages::EventQuery;
use crate::message::MessageResponder;

#[get("/event/{id}")]
async fn event(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let message = EventQuery { id };
    message.handle(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(event);
}
