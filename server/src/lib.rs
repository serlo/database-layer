use std::env;
use std::time::Duration;

use actix_web::web::Data;
use actix_web::App;
use dotenv::dotenv;
use regex::Regex;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::pool::Pool;
use sqlx::{MySql, MySqlPool};
use thiserror::Error;

pub mod alias;
pub mod datetime;
pub mod event;
pub mod instance;
pub mod message;
pub mod metadata;
pub mod navigation;
pub mod notification;
pub mod operation;
pub mod routes;
pub mod subject;
pub mod subscription;
pub mod thread;
pub mod user;
pub mod uuid;
pub mod vocabulary;

pub fn format_alias(prefix: Option<&str>, id: i32, suffix: Option<&str>) -> String {
    let prefix = prefix
        .map(|p| format!("/{}", slugify(p)))
        .unwrap_or_else(|| "".to_string());
    let suffix = suffix.map(slugify).unwrap_or_else(|| "".to_string());
    format!("{prefix}/{id}/{suffix}")
}

fn slugify(segment: &str) -> String {
    let segment = Regex::new(r#"['"`=+*&^%$#@!<>?]"#)
        .unwrap()
        .replace_all(segment, "");
    let segment = Regex::new(r"[\[\]{}() ,;:/|\-]+")
        .unwrap()
        .replace_all(segment.as_ref(), "-");
    segment.to_lowercase().trim_matches('-').to_string()
}

pub fn configure_app<T>(app: App<T>, pool: MySqlPool) -> App<T>
where
    T: actix_service::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Error = actix_web::Error,
        InitError = (),
    >,
{
    app.app_data(Data::new(pool)).configure(routes::init)
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
        .charset("utf8mb4");
    let pool = MySqlPoolOptions::new()
        .max_connections(database_max_connections)
        .acquire_timeout(Duration::from_secs(10 * 60))
        .connect_with(options)
        .await?;

    Ok(pool)
}

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Server error: {inner:?}.")]
    ServerError { inner: std::io::Error },
}

impl From<sqlx::Error> for ApplicationError {
    fn from(inner: sqlx::Error) -> Self {
        Self::DatabaseError { inner }
    }
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
