use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::messages::UuidQuery;
use super::model::{SetUuidStateError, SetUuidStatePayload, Uuid};

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
    match Uuid::set_uuid_state(payload.into_inner(), db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("/set-uuid-state: {:?}", e);
            match e {
                SetUuidStateError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().finish()
                }
                SetUuidStateError::EventError { .. } => {
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(uuid);
    cfg.service(set_state);
}
