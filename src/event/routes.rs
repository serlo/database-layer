use crate::event::model::Event;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/event/{id}")]
async fn event(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let result = Event::get_event(id, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            println!("Could not get event: {:?}", e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(event);
}
