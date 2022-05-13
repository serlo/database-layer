use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{Event, EventError};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventMessage {
    EventQuery(event_query::Payload),
    EventsQuery(events_query::Payload),
}

#[async_trait]
impl MessageResponder for EventMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            EventMessage::EventQuery(payload) => payload.handle("EventQuery", connection).await,
            EventMessage::EventsQuery(payload) => payload.handle("EventsQuery", connection).await,
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

pub mod events_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        after: Option<i32>,
        actor_id: Option<i32>,
        object_id: Option<i32>,
        instance: Option<Instance>,
        first: i32,
    }

    #[derive(Debug, Eq, PartialEq, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub events: Vec<Event>,
        pub has_next_page: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            if self.first > 10_000 {
                return Err(operation::Error::BadRequest {
                    reason: "parameter `first` is too high".to_string(),
                });
            }

            Ok(match connection {
                Connection::Pool(pool) => {
                    Event::fetch_events(
                        self.first,
                        self.after,
                        self.actor_id,
                        self.object_id,
                        self.instance.as_ref(),
                        pool,
                    )
                    .await?
                }
                Connection::Transaction(transaction) => {
                    Event::fetch_events(
                        self.first,
                        self.after,
                        self.actor_id,
                        self.object_id,
                        self.instance.as_ref(),
                        transaction,
                    )
                    .await?
                }
            })
        }
    }
}
