use actix_web::{get, web, Responder};
use sqlx::MySqlPool;

use super::messages::NavigationQuery;
use crate::instance::Instance;
use crate::message::MessageResponder;

#[get("/navigation/{instance}")]
async fn navigation(
    instance: web::Path<Instance>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let instance = instance.into_inner();
    let message = NavigationQuery { instance };
    message.handle(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(navigation);
}
