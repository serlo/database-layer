use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{Navigation, NavigationError};
use crate::instance::Instance;

#[get("/navigation/{instance}")]
async fn navigation(
    instance: web::Path<Instance>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let instance = instance.into_inner();
    match Navigation::fetch(instance.clone(), db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/navigation/{:?}: {:?}", instance, e);
            match e {
                NavigationError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(navigation);
}
