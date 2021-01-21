use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{License, LicenseError};

#[get("/license/{id}")]
async fn license(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    match License::fetch(id, db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/license/{:?}: {:?}", id, e);
            match e {
                LicenseError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                LicenseError::NotFound => HttpResponse::NotFound().json(None::<String>),
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(license);
}
