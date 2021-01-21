use actix_web::{get, App, HttpServer, Result};
use regex::Regex;

use serlo_org_database_layer::{
    alias, create_database_pool, event, health, license, navigation, notifications, subscriptions, threads, user, uuid,
};

#[get("/")]
async fn index() -> Result<String> {
    Ok("Ok".to_string())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let pool = create_database_pool().await?;

    println!("🚀 Server ready: http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(index)
            .configure(alias::init)
            .configure(event::init)
            .configure(health::init)
            .configure(license::init)
            .configure(navigation::init)
            .configure(notifications::init)
            .configure(subscriptions::init)
            .configure(threads::init)
            .configure(user::init)
            .configure(uuid::init)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
