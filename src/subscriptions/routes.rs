use crate::subscriptions::model::Subscriptions;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/subscriptions/{user_id}")]
async fn subscriptions(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let user_id = user_id.into_inner();
    let result = Subscriptions::get_subscriptions_for_user(user_id, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            println!("Could not get subscriptions for {}: {:?}", user_id, e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(subscriptions);
}
