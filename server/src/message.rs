use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::alias::AliasMessage;
use crate::event::EventMessage;
use crate::metadata::MetadataMessage;
use crate::navigation::NavigationMessage;
use crate::notification::NotificationMessage;
use crate::subject::SubjectsMessage;
use crate::subscription::SubscriptionMessage;
use crate::thread::ThreadMessage;
use crate::user::UserMessage;
use crate::uuid::{EntityMessage, PageMessage, TaxonomyTermMessage, UuidMessage};
use crate::vocabulary::VocabularyMessage;

/// A message responder maps the given message to a [`actix_web::HttpResponse`]
#[async_trait]
pub trait MessageResponder {
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse;
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    AliasMessage(AliasMessage),
    EntityMessage(EntityMessage),
    EventMessage(EventMessage),
    MetadataMessage(MetadataMessage),
    NavigationMessage(NavigationMessage),
    NotificationMessage(NotificationMessage),
    PageMessage(PageMessage),
    SubjectsMessage(SubjectsMessage),
    SubscriptionMessage(SubscriptionMessage),
    TaxonomyTermMessage(TaxonomyTermMessage),
    ThreadMessage(ThreadMessage),
    UserMessage(UserMessage),
    UuidMessage(UuidMessage),
    VocabularyMessage(VocabularyMessage),
}

#[async_trait]
impl MessageResponder for Message {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            Message::AliasMessage(message) => message.handle(acquire_from).await,
            Message::EntityMessage(message) => message.handle(acquire_from).await,
            Message::EventMessage(message) => message.handle(acquire_from).await,
            Message::MetadataMessage(message) => message.handle(acquire_from).await,
            Message::NavigationMessage(message) => message.handle(acquire_from).await,
            Message::NotificationMessage(message) => message.handle(acquire_from).await,
            Message::PageMessage(message) => message.handle(acquire_from).await,
            Message::SubjectsMessage(message) => message.handle(acquire_from).await,
            Message::SubscriptionMessage(message) => message.handle(acquire_from).await,
            Message::TaxonomyTermMessage(message) => message.handle(acquire_from).await,
            Message::ThreadMessage(message) => message.handle(acquire_from).await,
            Message::UserMessage(message) => message.handle(acquire_from).await,
            Message::UuidMessage(message) => message.handle(acquire_from).await,
            Message::VocabularyMessage(message) => message.handle(acquire_from).await,
        }
    }
}
