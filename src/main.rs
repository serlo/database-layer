use actix_web::{get, App, HttpServer, Result};

use serlo_org_database_layer::{configure_app, create_database_pool};

#[get("/")]
async fn index() -> Result<String> {
    Ok("Ok".to_string())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let pool = create_database_pool().await?;

    println!("🚀 Server ready: http://localhost:8080");

    HttpServer::new(move || configure_app(App::new(), pool.clone()).service(index))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}
