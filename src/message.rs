use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::uuid::UuidMessage;

/// A message responder maps the given message to a [`actix_web::HttpResponse`]
#[async_trait]
pub trait MessageResponder {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse;
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    UuidMessage(UuidMessage),
}

#[async_trait]
impl MessageResponder for Message {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            Message::UuidMessage(message) => message.handle(pool).await,
        }
    }
}
