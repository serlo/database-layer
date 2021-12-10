use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::alias::AliasMessage;
use crate::database::Connection;
use crate::event::EventMessage;
use crate::license::LicenseMessage;
use crate::metadata::MetadataMessage;
use crate::navigation::NavigationMessage;
use crate::notification::NotificationMessage;
use crate::subject::SubjectsMessage;
use crate::subscription::SubscriptionMessage;
use crate::thread::ThreadMessage;
use crate::user::UserMessage;
use crate::uuid::{EntityMessage, PageMessage, UuidMessage};
use crate::vocabulary::VocabularyMessage;

/// A message responder maps the given message to a [`actix_web::HttpResponse`]
#[async_trait]
pub trait MessageResponder {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse;
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    AliasMessage(AliasMessage),
    EntityMessage(EntityMessage),
    EventMessage(EventMessage),
    LicenseMessage(LicenseMessage),
    MetadataMessage(MetadataMessage),
    NavigationMessage(NavigationMessage),
    NotificationMessage(NotificationMessage),
    PageMessage(PageMessage),
    SubjectsMessage(SubjectsMessage),
    SubscriptionMessage(SubscriptionMessage),
    ThreadMessage(ThreadMessage),
    UserMessage(UserMessage),
    UuidMessage(UuidMessage),
    VocabularyMessage(VocabularyMessage),
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
            Message::MetadataMessage(message) => message.handle(connection).await,
            Message::NavigationMessage(message) => message.handle(connection).await,
            Message::NotificationMessage(message) => message.handle(connection).await,
            Message::PageMessage(message) => message.handle(connection).await,
            Message::SubjectsMessage(message) => message.handle(connection).await,
            Message::SubscriptionMessage(message) => message.handle(connection).await,
            Message::ThreadMessage(message) => message.handle(connection).await,
            Message::UserMessage(message) => message.handle(connection).await,
            Message::UuidMessage(message) => message.handle(connection).await,
            Message::VocabularyMessage(message) => message.handle(connection).await,
        }
    }
}
