use actix_web::{App, HttpServer, Result};

use serlo_org_database_layer::{configure_app, create_database_pool, ApplicationError};

#[actix_web::main]
async fn main() -> Result<(), ApplicationError> {
    let pool = create_database_pool().await?;

    println!("🚀 Server ready: http://localhost:8080");

    HttpServer::new(move || configure_app(App::new(), pool.clone()))
        .bind("0.0.0.0:8080")
        .map_err(|inner| ApplicationError::ServerError { inner })?
        .run()
        .await
        .map_err(|inner| ApplicationError::ServerError { inner })
}
