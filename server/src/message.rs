use actix_web::HttpResponse;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
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

/// A message responder maps the given message to a [`actix_web::HttpResponse`]
#[async_trait]
#[enum_dispatch]
pub trait MessageResponder {
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse;
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[enum_dispatch(MessageResponder)]
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
}
