use crate::notifications::model::{Notifications, NotificationsError};
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/notifications/{user_id}")]
async fn notifications(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let user_id = user_id.into_inner();
    let result = Notifications::get_notifications_for_user(user_id, db_pool.get_ref()).await;
    match result {
        Ok(user_array) => HttpResponse::Ok().json(user_array),
        Err(e) => {
            println!("Could not get notifications for {}: {:?}", user_id, e);
            match e.downcast_ref::<NotificationsError>() {
                Some(NotificationsError::NotFound { .. }) => {
                    HttpResponse::NotFound().json(None::<String>)
                }
                _ => HttpResponse::BadRequest().json(None::<String>),
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(notifications);
}
