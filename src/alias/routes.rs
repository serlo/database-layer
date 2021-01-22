use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{Alias, AliasError};

#[get("/alias/{instance}/{path:.*}")]
async fn alias(
    params: web::Path<(String, String)>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let (instance, path) = params.into_inner();
    match Alias::fetch(&path, &instance, db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/alias/{}/{}: {:?}", instance, path, e);
            match e {
                AliasError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
                AliasError::LegacyRoute => HttpResponse::NotFound().json(None::<String>),
                AliasError::NotFound => HttpResponse::NotFound().json(None::<String>),
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(alias);
}
