use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::Subscriptions;

#[get("/subscriptions/{user_id}")]
async fn subscriptions(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let user_id = user_id.into_inner();
    let result = Subscriptions::find_by_user_id(user_id, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("Could not get subscriptions for {}: {:?}", user_id, e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(subscriptions);
}
