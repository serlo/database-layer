use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::alias::AliasMessage;
use crate::event::EventMessage;
use crate::license::LicenseMessage;
use crate::navigation::NavigationMessage;
use crate::notification::NotificationMessage;
use crate::subscription::SubscriptionMessage;
use crate::uuid::UuidMessage;

/// A message responder maps the given message to a [`actix_web::HttpResponse`]
#[async_trait]
pub trait MessageResponder {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse;
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    AliasMessage(AliasMessage),
    EventMessage(EventMessage),
    LicenseMessage(LicenseMessage),
    NavigationMessage(NavigationMessage),
    NotificationMessage(NotificationMessage),
    SubscriptionMessage(SubscriptionMessage),
    UuidMessage(UuidMessage),
}

#[async_trait]
impl MessageResponder for Message {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            Message::AliasMessage(message) => message.handle(pool).await,
            Message::EventMessage(message) => message.handle(pool).await,
            Message::LicenseMessage(message) => message.handle(pool).await,
            Message::NavigationMessage(message) => message.handle(pool).await,
            Message::NotificationMessage(message) => message.handle(pool).await,
            Message::SubscriptionMessage(message) => message.handle(pool).await,
            Message::UuidMessage(message) => message.handle(pool).await,
        }
    }
}
