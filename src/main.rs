use actix_web::{get, web, App, HttpServer, Result};
use dotenv::dotenv;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};

mod uuid;

#[get("/uuid/{uuid}")]
async fn hello(web::Path(uuid): web::Path<u32>) -> Result<String> {
    Ok(format!("Hello {}", uuid))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // TODO: we can't use DATABASE_URL correctly because we need to override charset.
    let options = MySqlConnectOptions::new()
        .host("localhost")
        .username("root")
        .password("secret")
        .database("serlo")
        .charset("latin1");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    HttpServer::new(move || App::new().data(pool.clone()).configure(uuid::init))
        .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}
