use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::model::{Event, EventError, Events};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventMessage {
    EventQuery(EventQuery),
    EventsQuery(EventsQuery),
}

#[async_trait]
impl MessageResponder for EventMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            EventMessage::EventQuery(message) => message.handle(connection).await,
            EventMessage::EventsQuery(message) => message.handle(connection).await,
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
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let event = match connection {
            Connection::Pool(pool) => Event::fetch(self.id, pool).await,
            Connection::Transaction(transaction) => {
                Event::fetch_via_transaction(self.id, transaction).await
            }
        };
        match event {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/event/{}: {:?}", self.id, e);
                match e {
                    EventError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EventError::InvalidType => HttpResponse::NotFound().json(&None::<String>),
                    EventError::InvalidInstance => HttpResponse::InternalServerError().finish(),
                    EventError::MissingRequiredField => {
                        HttpResponse::NotFound().json(&None::<String>)
                    }
                    EventError::NotFound => HttpResponse::NotFound().json(&None::<String>),
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsQuery {
    after: Option<i32>,
    actor_id: Option<i32>,
    object_id: Option<i32>,
    first: i32,
}

#[async_trait]
impl MessageResponder for EventsQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        if self.first > 10_000 {
            return HttpResponse::BadRequest().json(json!({
                "success": false,
                "reason": "parameter `first` is too high",
            }));
        }
        let events = match connection {
            Connection::Pool(pool) => {
                Events::fetch(self.after, self.actor_id, self.object_id, self.first, pool).await
            }
            Connection::Transaction(transaction) => {
                Events::fetch_via_transaction(
                    self.after,
                    self.actor_id,
                    self.object_id,
                    self.first,
                    transaction,
                )
                .await
            }
        };
        match events {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/events: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
