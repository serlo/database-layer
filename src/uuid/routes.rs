use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{SetUuidStateError, SetUuidStatePayload, Uuid, UuidError};

#[get("/uuid/{id}")]
async fn uuid(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    match Uuid::fetch(id, db_pool.get_ref()).await {
        Ok(uuid) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(uuid),
        Err(e) => {
            println!("/uuid/{}: {:?}", id, e);
            match e {
                UuidError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                UuidError::UnsupportedDiscriminator { .. } => {
                    HttpResponse::NotFound().json(None::<String>)
                }
                UuidError::UnsupportedEntityType { .. } => {
                    HttpResponse::NotFound().json(None::<String>)
                }
                UuidError::UnsupportedEntityRevisionType { .. } => {
                    HttpResponse::NotFound().json(None::<String>)
                }
                UuidError::EntityMissingRequiredParent => {
                    HttpResponse::NotFound().json(None::<String>)
                }
                UuidError::NotFound => HttpResponse::NotFound().json(None::<String>),
            }
        }
    }
}

#[post("/set-uuid-state")]
async fn set_state(
    payload: web::Json<SetUuidStatePayload>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    match Uuid::set_uuid_state(payload.into_inner(), db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(vec![1 as i32]),
        Err(e) => {
            println!("/set-uuid-state/: {:?}", e);
            match e {
                SetUuidStateError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                SetUuidStateError::EventError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(uuid);
    cfg.service(set_state);
}
