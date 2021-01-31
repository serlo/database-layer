use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use crate::message::{Message, MessageResponder};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/")]
async fn handle_message(
    payload: web::Json<Message>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let message = payload.into_inner();
    message.handle(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(handle_message);
}
