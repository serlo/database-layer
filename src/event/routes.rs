use actix_web::{get, web, Responder};
use sqlx::MySqlPool;

use super::messages::EventQuery;
use crate::database::Connection;
use crate::message::MessageResponderNew;

#[get("/event/{id}")]
async fn event(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let message = EventQuery { id };
    let connection = Connection::Pool(db_pool.get_ref());
    message.handle_new(connection).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(event);
}
