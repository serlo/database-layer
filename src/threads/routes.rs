use crate::threads::model::Threads;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/threads/{id}")]
async fn threads(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let result = Threads::get_thread_ids(id, db_pool.get_ref()).await;
    match result {
        Ok(user_array) => HttpResponse::Ok().json(user_array),
        Err(e) => {
            println!("Could not get active authors: {:?}", e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(threads);
}
