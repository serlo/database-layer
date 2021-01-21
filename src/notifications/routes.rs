use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::Notifications;

#[get("/notifications/{user_id}")]
async fn find(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let notifications = user_id.into_inner();
    let result = Notifications::fetch(notifications, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!(
                "Could not get notifications of user {}: {:?}",
                notifications, e
            );
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find);
}
