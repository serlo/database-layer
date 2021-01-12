use actix_web::{App, HttpServer};
use dotenv::dotenv;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};

#[macro_use]
extern crate dotenv_codegen;

mod uuid;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let options = MySqlConnectOptions::new()
        .host(dotenv!("DATABASE_HOST"))
        .port(dotenv!("DATABASE_PORT").parse().unwrap())
        .username(dotenv!("DATABASE_USERNAME"))
        .password(dotenv!("DATABASE_PASSWORD"))
        .database(dotenv!("DATABASE_DATABASE"))
        .charset("latin1");
    let pool = MySqlPoolOptions::new()
        .max_connections(dotenv!("DATABASE_MAX_CONNECTIONS").parse().unwrap())
        .connect_with(options)
        .await?;

    println!("🚀 Server ready: http://localhost:8080");

    HttpServer::new(move || App::new().data(pool.clone()).configure(uuid::init))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}
