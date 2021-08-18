use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use thiserror::Error;

use crate::alias::AliasMessage;
use crate::database::Connection;
use crate::event::EventMessage;
use crate::license::LicenseMessage;
use crate::navigation::NavigationMessage;
use crate::notification::NotificationMessage;
use crate::subject::SubjectsMessage;
use crate::subscription::SubscriptionMessage;
use crate::thread::ThreadMessage;
use crate::user::UserMessages;
use crate::uuid::{EntityMessage, PageMessage, UuidMessage};

/// A message responder maps the given message to a [`actix_web::HttpResponse`]
#[async_trait]
pub trait MessageResponder {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse;
}

#[derive(Debug, Error)]
pub enum OperationError {
    #[error("BadRequest: {0}")]
    BadRequest(String),
    #[error("InternalServerError: {0}")]
    InternalServerError(Box<dyn std::error::Error>),
}

impl From<sqlx::Error> for OperationError {
    fn from(error: sqlx::Error) -> Self {
        OperationError::InternalServerError(Box::new(error))
    }
}

pub type OperationResult<T> = Result<T, OperationError>;

#[async_trait]
pub trait Operation {
    type Output: Serialize;

    async fn execute(&self, connection: Connection<'_, '_>) -> OperationResult<Self::Output>;

    #[allow(clippy::async_yields_async)]
    async fn handle(&self, operation_type: &str, connection: Connection<'_, '_>) -> HttpResponse {
        match &self.execute(connection).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),

            Err(error) => {
                println!("{}: {}", operation_type, error);

                match error {
                    OperationError::BadRequest(reason) => HttpResponse::BadRequest()
                        .content_type("application/json; charset=utf-8")
                        .json(json!({ "success": false, "reason": reason })),
                    OperationError::InternalServerError(_) => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    AliasMessage(AliasMessage),
    EntityMessage(EntityMessage),
    EventMessage(EventMessage),
    LicenseMessage(LicenseMessage),
    NavigationMessage(NavigationMessage),
    NotificationMessage(NotificationMessage),
    PageMessage(PageMessage),
    SubjectsMessage(SubjectsMessage),
    SubscriptionMessage(SubscriptionMessage),
    ThreadMessage(ThreadMessage),
    UserMessage(UserMessages),
    UuidMessage(UuidMessage),
}

#[async_trait]
impl MessageResponder for Message {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            Message::AliasMessage(message) => message.handle(connection).await,
            Message::EntityMessage(message) => message.handle(connection).await,
            Message::EventMessage(message) => message.handle(connection).await,
            Message::LicenseMessage(message) => message.handle(connection).await,
            Message::NavigationMessage(message) => message.handle(connection).await,
            Message::NotificationMessage(message) => message.handle(connection).await,
            Message::PageMessage(message) => message.handle(connection).await,
            Message::SubjectsMessage(message) => message.handle(connection).await,
            Message::SubscriptionMessage(message) => message.handle(connection).await,
            Message::ThreadMessage(message) => message.handle(connection).await,
            Message::UserMessage(message) => message.handle(connection).await,
            Message::UuidMessage(message) => message.handle(connection).await,
        }
    }
}
