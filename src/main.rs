use std::env;

use actix_web::{App, HttpServer, Result};

use serlo_org_database_layer::{configure_app, create_database_pool, ApplicationError};

#[actix_web::main]
async fn main() -> Result<(), ApplicationError> {
    let sentry_dsn = env::var("SENTRY_DSN").ok();
    let _guard = sentry::init((
        sentry_dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    env::set_var("RUST_BACKTRACE", "1");

    let pool = create_database_pool().await?;

    println!("🚀 Server ready: http://localhost:8080");

    HttpServer::new(move || {
        let app = App::new().wrap(sentry_actix::Sentry::new());
        configure_app(app, pool.clone())
    })
    .bind("0.0.0.0:8080")
    .map_err(|inner| ApplicationError::ServerError { inner })?
    .run()
    .await
    .map_err(|inner| ApplicationError::ServerError { inner })
}
