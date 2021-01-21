use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::Alias;

#[get("/alias/{instance}/{path:.*}")]
async fn alias(
    params: web::Path<(String, String)>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    let (instance, path) = params.into_inner();
    let result = Alias::fetch(&path, &instance, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!(
                "Could not resolve alias {} in instance {}: {:?}",
                path, instance, e
            );
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(alias);
}
