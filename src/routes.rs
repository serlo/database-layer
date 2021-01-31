use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::uuid::UuidMessage;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum Message {
    UuidMessage(UuidMessage),
}

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
    let pool = db_pool.get_ref();
    match message {
        Message::UuidMessage(message) => message.handle(pool).await,
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(handle_message);
}
