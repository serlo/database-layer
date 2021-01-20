use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{Uuid, UuidError};

#[get("/uuid/{id}")]
async fn uuid(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let result = Uuid::find_by_id(id, db_pool.get_ref()).await;
    match result {
        Ok(uuid) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(uuid),
        Err(e) => {
            println!("UUID {} failed: {:?}", id, e);
            match e.downcast_ref::<UuidError>() {
                Some(UuidError::UnsupportedDiscriminator { .. }) => {
                    HttpResponse::BadRequest().json(None::<String>)
                }
                Some(UuidError::NotFound { .. }) => HttpResponse::NotFound().json(None::<String>),
                _ => HttpResponse::BadRequest().json(None::<String>),
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(uuid);
}
