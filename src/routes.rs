use actix_web::{get, post, web, HttpRequest, HttpResponse};
use sqlx::MySqlPool;

use crate::database::Connection;
use crate::message::{Message, MessageResponder};

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/")]
async fn handle_message(
    req: HttpRequest,
    payload: web::Json<Message>,
    db_pool: web::Data<MySqlPool>,
) -> HttpResponse {
    let rollback = req
        .headers()
        .get("Rollback")
        .map_or(false, |value| match value.to_str() {
            Ok("true") => true,
            _ => false,
        });
    let message = payload.into_inner();
    let pool = db_pool.get_ref();

    if rollback {
        let mut transaction = pool.begin().await.expect("Failed to begin transaction.");
        let connection = Connection::Transaction(&mut transaction);
        let response = message.handle(connection).await;
        transaction
            .rollback()
            .await
            .expect("Failed to roll back transaction.");
        response
    } else {
        let connection = Connection::Pool(pool);
        message.handle(connection).await
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(handle_message);
}
