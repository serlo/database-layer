use crate::threads::model::Threads;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/threads/{id}")]
async fn find(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    let result = Threads::find_by_id(id, db_pool.get_ref()).await;
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            println!("Could not get threads: {:?}", e);
            HttpResponse::BadRequest().json(None::<String>)
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find);
}
