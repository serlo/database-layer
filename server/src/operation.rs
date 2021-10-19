use crate::database::Connection;
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::{json, Value};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("BadRequest: {reason:?}")]
    BadRequest { reason: String },
    #[error("InternalServerError: {error:?}")]
    InternalServerError { error: Box<dyn std::error::Error> },
    #[error("Requested value could not be found.")]
    NotFoundError,
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::InternalServerError {
            error: Box::new(error),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Operation {
    type Output: Serialize;

    async fn execute(&self, connection: Connection<'_, '_>) -> Result<Self::Output>;

    #[allow(clippy::async_yields_async)]
    async fn handle(&self, operation_type: &str, connection: Connection<'_, '_>) -> HttpResponse {
        match &self.execute(connection).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),

            Err(error) => {
                println!("{}: {}", operation_type, error);

                match error {
                    Error::NotFoundError => HttpResponse::NotFound()
                        .content_type("application/json; charset=utf8")
                        .json(Value::Null),
                    Error::BadRequest { reason } => HttpResponse::BadRequest()
                        .content_type("application/json; charset=utf-8")
                        .json(json!({ "success": false, "reason": reason })),
                    Error::InternalServerError { error: _ } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}
