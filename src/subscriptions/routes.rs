use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{Subscriptions, SubscriptionsError};

#[get("/subscriptions/{user_id}")]
async fn subscriptions(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let user_id = user_id.into_inner();
    match Subscriptions::fetch(user_id, db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/subscriptions/{:?}: {:?}", user_id, e);
            match e {
                SubscriptionsError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(subscriptions);
}
