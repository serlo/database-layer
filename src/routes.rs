use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

#[derive(Deserialize, Serialize)]
struct MessagePayload {}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/")]
async fn handle_message(
    _payload: web::Json<MessagePayload>,
    _db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    HttpResponse::Ok()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(handle_message);
}
