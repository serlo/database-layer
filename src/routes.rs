use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use crate::database::Connection;
use crate::message::{Message, MessageResponderNew};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/")]
async fn handle_message(
    payload: web::Json<Message>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let pool = db_pool.get_ref();
    let connection = Connection::Pool(pool);
    let message = payload.into_inner();
    message.handle_new(connection).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(handle_message);
}
