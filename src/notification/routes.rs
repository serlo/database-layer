use actix_web::{get, post, web, Responder};
use sqlx::MySqlPool;

use super::messages::{NotificationSetStateMutation, NotificationsQuery};
use super::model::SetNotificationStatePayload;
use crate::database::Connection;
use crate::message::MessageResponderNew;

#[get("/notifications/{user_id}")]
async fn notifications(user_id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let user_id = user_id.into_inner();
    let message = NotificationsQuery { user_id };
    let connection = Connection::Pool(db_pool.get_ref());
    message.handle_new(connection).await
}

#[post("/set-notification-state")]
async fn set_state(
    payload: web::Json<SetNotificationStatePayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let payload = payload.into_inner();
    let message = NotificationSetStateMutation {
        ids: payload.ids.clone(),
        user_id: payload.user_id,
        unread: payload.unread,
    };
    let connection = Connection::Pool(db_pool.get_ref());
    message.handle_new(connection).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(notifications);
    cfg.service(set_state);
}
