use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::model::{Event, EventError, Events};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventMessage {
    EventQuery(event_query::Payload),
    EventsQuery(EventsQuery),
}

#[async_trait]
impl MessageResponder for EventMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            EventMessage::EventQuery(payload) => payload.handle("EventQuery", connection).await,
            EventMessage::EventsQuery(message) => message.handle(connection).await,
        }
    }
}

pub mod event_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Event;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => Event::fetch(self.id, pool).await,
                Connection::Transaction(transaction) => {
                    Event::fetch_via_transaction(self.id, transaction).await
                }
            }
            .map_err(|e| match e {
                EventError::InvalidType
                | EventError::MissingRequiredField
                | EventError::NotFound => operation::Error::NotFoundError,
                _ => operation::Error::InternalServerError { error: Box::new(e) },
            })?)
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsQuery {
    after: Option<i32>,
    actor_id: Option<i32>,
    object_id: Option<i32>,
    instance: Option<Instance>,
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
                Events::fetch(
                    self.first,
                    self.after,
                    self.actor_id,
                    self.object_id,
                    self.instance.as_ref(),
                    pool,
                )
                .await
            }
            Connection::Transaction(transaction) => {
                Events::fetch_via_transaction(
                    self.first,
                    self.after,
                    self.actor_id,
                    self.object_id,
                    self.instance.as_ref(),
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
