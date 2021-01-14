use crate::uuid::model::{Uuid, UuidError};
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/uuid/{id}")]
async fn find(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let result = Uuid::find_by_id(id, db_pool.get_ref()).await;
    match result {
        Ok(uuid) => HttpResponse::Ok().json(uuid),
        Err(e) => {
            println!("UUID {} failed: {:?}", id, e);
            match e {
                UuidError::InvalidDiscriminator { .. } => {
                    HttpResponse::BadRequest().json(None::<String>)
                }
                UuidError::NotFound => HttpResponse::NotFound().json(None::<String>),
                UuidError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find);
}
