use crate::navigation::model::Navigation;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/navigation/{instance}")]
async fn navigation(instance: web::Path<String>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let instance = instance.into_inner();
    let result = Navigation::find_navigation_by_instance(&instance, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            println!(
                "Could not get navigation for instance {}: {:?}",
                instance, e
            );
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(navigation);
}
