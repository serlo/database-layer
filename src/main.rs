use actix_web::{get, web, App, HttpServer, Result};
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

mod uuid;

#[get("/uuid/{uuid}")]
async fn hello(web::Path(uuid): web::Path<u32>) -> Result<String> {
    Ok(format!("Hello {}", uuid))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    HttpServer::new(move || App::new().data(pool.clone()).configure(uuid::init))
        .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}
