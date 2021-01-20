use actix_web::{get, App, HttpServer, Result};
use dotenv::dotenv;
use regex::Regex;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::env;

use serlo_org_database_layer::{
    alias, event, health, license, navigation, notifications, subscriptions, threads, user, uuid,
};

#[get("/")]
async fn index() -> Result<String> {
    Ok("Ok".to_string())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let database_max_connections: u32 = env::var("DATABASE_MAX_CONNECTIONS")
        .expect("DATABASE_MAX_CONNECTIONS is not set.")
        .parse()
        .unwrap();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set.");
    let re = Regex::new(
        r"^mysql://(?P<username>.+):(?P<password>.+)@(?P<host>.+):(?P<port>\d+)/(?P<database>.+)$",
    )
    .unwrap();
    let captures = re.captures(&database_url).unwrap();
    let username = captures.name("username").unwrap().as_str();
    let password = captures.name("password").unwrap().as_str();
    let host = captures.name("host").unwrap().as_str();
    let port: u16 = captures.name("port").unwrap().as_str().parse().unwrap();
    let database = captures.name("database").unwrap().as_str();

    let options = MySqlConnectOptions::new()
        .host(host)
        .port(port)
        .username(username)
        .password(password)
        .database(database)
        .charset("latin1");
    let pool = MySqlPoolOptions::new()
        .max_connections(database_max_connections)
        .connect_with(options)
        .await?;

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
