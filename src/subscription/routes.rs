use actix_web::{get, web, Responder};
use sqlx::MySqlPool;

use super::messages::SubscriptionsQuery;
use crate::database::Connection;
use crate::message::MessageResponder;

#[get("/subscriptions/{user_id}")]
async fn subscriptions(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let user_id = user_id.into_inner();
    let message = SubscriptionsQuery { user_id };
    let connection = Connection::Pool(db_pool.get_ref());
    message.handle(connection).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(subscriptions);
}
