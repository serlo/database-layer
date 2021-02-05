use actix_web::{get, web, Responder};
use sqlx::MySqlPool;

use super::messages::LicenseQuery;
use crate::message::MessageResponder;

#[get("/license/{id}")]
async fn license(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let message = LicenseQuery { id };
    message.handle(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(license);
}
