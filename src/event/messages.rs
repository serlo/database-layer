use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{Event, EventError};
use crate::database::ConnectionLike;
use crate::message::{MessageResponder, MessageResponderNew};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventMessage {
    EventQuery(EventQuery),
}

#[async_trait]
impl MessageResponder for EventMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            EventMessage::EventQuery(message) => message.handle(pool).await,
        }
    }
}

#[async_trait]
impl MessageResponderNew for EventMessage {
    async fn handle_new(&self, connection: ConnectionLike<'_, '_>) -> HttpResponse {
        match self {
            EventMessage::EventQuery(message) => message.handle_new(connection).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventQuery {
    pub id: i32,
}

#[async_trait]
impl MessageResponder for EventQuery {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match Event::fetch(self.id, pool).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/event/{}: {:?}", self.id, e);
                match e {
                    EventError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EventError::InvalidType => HttpResponse::NotFound().json(None::<String>),
                    EventError::InvalidInstance => HttpResponse::InternalServerError().finish(),
                    EventError::MissingRequiredField => {
                        HttpResponse::NotFound().json(None::<String>)
                    }
                    EventError::NotFound => HttpResponse::NotFound().json(None::<String>),
                }
            }
        }
    }
}

#[async_trait]
impl MessageResponderNew for EventQuery {
    async fn handle_new(&self, connection: ConnectionLike<'_, '_>) -> HttpResponse {
        let event = match connection {
            ConnectionLike::Pool(pool) => Event::fetch(self.id, pool).await,
            ConnectionLike::Transaction(transaction) => {
                Event::fetch_via_transaction(self.id, transaction).await
            }
        };
        match event {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/event/{}: {:?}", self.id, e);
                match e {
                    EventError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EventError::InvalidType => HttpResponse::NotFound().json(None::<String>),
                    EventError::InvalidInstance => HttpResponse::InternalServerError().finish(),
                    EventError::MissingRequiredField => {
                        HttpResponse::NotFound().json(None::<String>)
                    }
                    EventError::NotFound => HttpResponse::NotFound().json(None::<String>),
                }
            }
        }
    }
}
