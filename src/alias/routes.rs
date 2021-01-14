use crate::alias::model::Alias;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/alias/{alias}")]
async fn alias(alias: web::Path<String>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let alias = alias.into_inner();
    let result = Alias::get_alias_data(&alias, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            println!("Could not get data for alias {}: {:?}", alias, e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(alias);
}
