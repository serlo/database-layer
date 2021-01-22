use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{Notifications, NotificationsError};

#[get("/notifications/{user_id}")]
async fn notifications(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let user_id = user_id.into_inner();
    match Notifications::fetch(user_id, db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/notifications/{:?}: {:?}", user_id, e);
            match e {
                NotificationsError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(notifications);
}
