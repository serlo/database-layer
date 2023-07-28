use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{Event, EventError};
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventMessage {
    EventQuery(event_query::Payload),
    EventsQuery(events_query::Payload),
}

#[async_trait]
impl MessageResponder for EventMessage {
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            EventMessage::EventQuery(payload) => payload.handle(acquire_from).await,
            EventMessage::EventsQuery(payload) => payload.handle(acquire_from).await,
        }
    }
}

pub mod event_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Event;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Event::fetch(self.id, acquire_from)
                .await
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

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub after: Option<i32>,
        pub actor_id: Option<i32>,
        pub object_id: Option<i32>,
        pub instance: Option<Instance>,
        pub first: i32,
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            if self.first > 10_000 {
                return Err(operation::Error::BadRequest {
                    reason: "parameter `first` is too high".to_string(),
                });
            }

            Ok(Event::fetch_events(self, acquire_from).await?)
        }
    }
}
