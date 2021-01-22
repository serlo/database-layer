use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{Event, EventError};

#[get("/event/{id}")]
async fn event(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    match Event::fetch(id, db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/event/{}: {:?}", id, e);
            match e {
                EventError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                EventError::InvalidType => HttpResponse::NotFound().json(None::<String>),
                EventError::MissingRequiredField => HttpResponse::NotFound().json(None::<String>),
                EventError::NotFound => HttpResponse::NotFound().json(None::<String>),
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(event);
}
