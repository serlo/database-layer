use std::env;

use actix_service::ServiceFactory;
use actix_web::dev::{MessageBody, ServiceRequest, ServiceResponse};
use actix_web::{App, Error};
use dotenv::dotenv;
use regex::Regex;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::pool::Pool;
use sqlx::{MySql, MySqlPool};
use thiserror::Error;

pub mod alias;
pub mod database;
pub mod datetime;
pub mod event;
pub mod health;
pub mod license;
pub mod navigation;
pub mod notifications;
pub mod subscriptions;
pub mod threads;
pub mod user;
pub mod uuid;

pub fn format_alias(prefix: Option<&str>, id: i32, suffix: Option<&str>) -> String {
    let prefix = prefix
        .map(|p| format!("/{}", slugify(p)))
        .unwrap_or_else(|| "".to_string());
    let suffix = suffix.map(slugify).unwrap_or_else(|| "".to_string());
    format!("{}/{}/{}", prefix, id, suffix)
}

fn slugify(segment: &str) -> String {
    let segment = Regex::new(r#"['"`=+*&^%$#@!<>?]"#)
        .unwrap()
        .replace_all(&segment, "");
    let segment = Regex::new(r"[\[\]{}() ,;:/|\-]+")
        .unwrap()
        .replace_all(&segment, "-");
    segment.to_lowercase().trim_matches('-').to_string()
}

pub fn configure_app<T, B>(app: App<T, B>, pool: MySqlPool) -> App<T, B>
where
    B: MessageBody,
    T: ServiceFactory<
        Config = (),
        Request = ServiceRequest,
        Response = ServiceResponse<B>,
        Error = Error,
        InitError = (),
    >,
{
    app.data(pool)
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
}

pub async fn create_database_pool() -> Result<Pool<MySql>, ApplicationError> {
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
    MySqlPoolOptions::new()
        .max_connections(database_max_connections)
        .connect_with(options)
        .await
        .map_err(|inner| ApplicationError::DatabaseError { inner })
}

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Server error: {inner:?}.")]
    ServerError { inner: std::io::Error },
}

#[cfg(test)]
mod test {
    use super::slugify;

    #[test]
    fn format_alias_double_dash() {
        assert_eq!(
            slugify("Flächen- und Volumenberechnung mit Integralen"),
            "flächen-und-volumenberechnung-mit-integralen"
        )
    }
}
