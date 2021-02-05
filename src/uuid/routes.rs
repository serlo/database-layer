use actix_web::{get, post, web, Responder};
use sqlx::MySqlPool;

use super::messages::{UuidQuery, UuidSetStateMutation};
use super::model::SetUuidStatePayload;
use crate::message::MessageResponder;

#[get("/uuid/{id}")]
async fn uuid(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let message = UuidQuery { id };
    message.handle(db_pool.get_ref()).await
}

#[post("/set-uuid-state")]
async fn set_state(
    payload: web::Json<SetUuidStatePayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let payload = payload.into_inner();
    let message = UuidSetStateMutation {
        ids: payload.ids.clone(),
        user_id: payload.user_id,
        trashed: payload.trashed,
    };
    message.handle(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(uuid);
    cfg.service(set_state);
}
