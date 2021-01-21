use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;

use crate::format_datetime;

use super::checkout_revision::CheckoutRevision;
use super::create_comment::CreateComment;
use super::create_entity::CreateEntity;
use super::create_entity_link::CreateEntityLink;
use super::create_entity_revision::CreateEntityRevision;
use super::create_taxonomy_link::CreateTaxonomyLink;
use super::create_taxonomy_term::CreateTaxonomyTerm;
use super::create_thread::CreateThread;
use super::reject_revision::RejectRevision;
use super::remove_entity_link::RemoveEntityLink;
use super::remove_taxonomy_link::RemoveTaxonomyLink;
use super::set_license::SetLicense;
use super::set_taxonomy_parent::SetTaxonomyParent;
use super::set_taxonomy_term::SetTaxonomyTerm;
use super::set_thread_state::SetThreadState;
use super::set_uuid_state::SetUuidState;
use super::unsupported::Unsupported;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Event {
    SetThreadState(SetThreadState),
    CreateComment(CreateComment),
    CreateThread(CreateThread),
    CreateEntity(CreateEntity),
    SetLicense(SetLicense),
    CreateEntityLink(CreateEntityLink),
    RemoveEntityLink(RemoveEntityLink),
    CreateEntityRevision(CreateEntityRevision),
    CheckoutRevision(CheckoutRevision),
    RejectRevision(RejectRevision),
    CreateTaxonomyLink(CreateTaxonomyLink),
    RemoveTaxonomyLink(RemoveTaxonomyLink),
    CreateTaxonomyTerm(CreateTaxonomyTerm),
    SetTaxonomyTerm(SetTaxonomyTerm),
    SetTaxonomyParent(SetTaxonomyParent),
    SetUuidState(SetUuidState),
    Unsupported(Unsupported),
}

#[derive(Deserialize, Serialize)]
pub enum EventType {
    #[serde(rename(
        serialize = "SetThreadStateNotificationEvent",
        deserialize = "discussion/comment/archive",
        deserialize = "discussion/comment/restore"
    ))]
    SetThreadState,
    #[serde(rename(
        serialize = "CreateCommentNotificationEvent",
        deserialize = "discussion/comment/create"
    ))]
    CreateComment,
    #[serde(rename(
        serialize = "CreateThreadNotificationEvent",
        deserialize = "discussion/create"
    ))]
    CreateThread,
    #[serde(rename(
        serialize = "CreateEntityNotificationEvent",
        deserialize = "entity/create"
    ))]
    CreateEntity,
    #[serde(rename(
        serialize = "SetLicenseNotificationEvent",
        deserialize = "license/object/set"
    ))]
    SetLicense,
    #[serde(rename(
        serialize = "CreateEntityLinkNotificationEvent",
        deserialize = "entity/link/create"
    ))]
    CreateEntityLink,
    #[serde(rename(
        serialize = "RemoveEntityLinkNotificationEvent",
        deserialize = "entity/link/remove"
    ))]
    RemoveEntityLink,
    #[serde(rename(
        serialize = "CreateEntityRevisionNotificationEvent",
        deserialize = "entity/revision/add"
    ))]
    CreateEntityRevision,
    #[serde(rename(
        serialize = "CheckoutRevisionNotificationEvent",
        deserialize = "entity/revision/checkout"
    ))]
    CheckoutRevision,
    #[serde(rename(
        serialize = "RejectRevisionNotificationEvent",
        deserialize = "entity/revision/reject"
    ))]
    RejectRevision,
    #[serde(rename(
        serialize = "CreateTaxonomyLinkNotificationEvent",
        deserialize = "taxonomy/term/associate"
    ))]
    CreateTaxonomyLink,
    #[serde(rename(
        serialize = "RemoveTaxonomyLinkNotificationEvent",
        deserialize = "taxonomy/term/dissociate"
    ))]
    RemoveTaxonomyLink,
    #[serde(rename(
        serialize = "CreateTaxonomyTermNotificationEvent",
        deserialize = "taxonomy/term/create"
    ))]
    CreateTaxonomyTerm,
    #[serde(rename(
        serialize = "SetTaxonomyTermNotificationEvent",
        deserialize = "taxonomy/term/update"
    ))]
    SetTaxonomyTerm,
    #[serde(rename(
        serialize = "SetTaxonomyParentNotificationEvent",
        deserialize = "taxonomy/term/parent/change"
    ))]
    SetTaxonomyParent,
    #[serde(rename(
        serialize = "SetUuidStateNotificationEvent",
        deserialize = "uuid/restore",
        deserialize = "uuid/trash"
    ))]
    SetUuidState,
    #[serde(rename(
        serialize = "UnsupportedNotificationEvent",
        deserialize = "discussion/restore"
    ))]
    Unsupported,
}

impl std::str::FromStr for EventType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEvent {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EventType,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub actor_id: i32,
    pub object_id: i32,

    #[serde(skip)]
    pub parameter_uuid_id: i32,
    #[serde(skip)]
    pub name: String,
}

impl Event {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Event> {
        let event = sqlx::query!(
            r#"
                SELECT e.id, e.actor_id, e.uuid_id, e.date, i.subdomain, ev.name, id.uuid_id AS parameter_uuid_id
                    FROM event_log e
                    LEFT JOIN event_parameter p ON e.id = p.log_id
                    LEFT JOIN event_parameter_uuid id ON p.id = id.event_parameter_id
                    JOIN instance i ON e.instance_id = i.id
                    JOIN event ev ON e.event_id = ev.id
                    WHERE e.id = ?
            "#,
            id
        )
            .fetch_one(pool)
            .await?;

        let name = event.name;
        let uuid_id = event.uuid_id as i32;
        let parameter_uuid_id = event
            .parameter_uuid_id
            .map(|id| id as i32)
            .unwrap_or(uuid_id);

        let abstract_event = AbstractEvent {
            __typename: name.parse::<EventType>()?,
            id: event.id as i32,
            instance: event.subdomain.to_string(),
            date: format_datetime(&event.date),
            actor_id: event.actor_id as i32,
            object_id: uuid_id,
            parameter_uuid_id,
            name,
        };

        let event = match abstract_event.name.as_str().parse()? {
            EventType::SetThreadState => {
                Event::SetThreadState(SetThreadState::fetch(abstract_event).await?)
            }
            EventType::CreateComment => {
                Event::CreateComment(CreateComment::fetch(abstract_event).await?)
            }
            EventType::CreateThread => {
                Event::CreateThread(CreateThread::fetch(abstract_event).await?)
            }
            EventType::CreateEntity => {
                Event::CreateEntity(CreateEntity::fetch(abstract_event).await?)
            }
            EventType::SetLicense => Event::SetLicense(SetLicense::fetch(abstract_event).await?),
            EventType::CreateEntityLink => {
                Event::CreateEntityLink(CreateEntityLink::fetch(abstract_event).await?)
            }
            EventType::RemoveEntityLink => {
                Event::RemoveEntityLink(RemoveEntityLink::fetch(abstract_event).await?)
            }
            EventType::CreateEntityRevision => {
                Event::CreateEntityRevision(CreateEntityRevision::fetch(abstract_event).await?)
            }
            EventType::CheckoutRevision => {
                Event::CheckoutRevision(CheckoutRevision::fetch(abstract_event, pool).await?)
            }
            EventType::RejectRevision => {
                Event::RejectRevision(RejectRevision::fetch(abstract_event, pool).await?)
            }
            EventType::CreateTaxonomyLink => {
                Event::CreateTaxonomyLink(CreateTaxonomyLink::fetch(abstract_event).await?)
            }
            EventType::RemoveTaxonomyLink => {
                Event::RemoveTaxonomyLink(RemoveTaxonomyLink::fetch(abstract_event).await?)
            }
            EventType::CreateTaxonomyTerm => {
                Event::CreateTaxonomyTerm(CreateTaxonomyTerm::fetch(abstract_event).await?)
            }
            EventType::SetTaxonomyTerm => {
                Event::SetTaxonomyTerm(SetTaxonomyTerm::fetch(abstract_event).await?)
            }
            EventType::SetTaxonomyParent => {
                Event::SetTaxonomyParent(SetTaxonomyParent::fetch(abstract_event, pool).await?)
            }
            EventType::SetUuidState => {
                Event::SetUuidState(SetUuidState::fetch(abstract_event).await?)
            }
            EventType::Unsupported => Event::Unsupported(Unsupported::fetch(abstract_event).await?),
        };

        Ok(event)
    }
}

pub async fn fetch_uuid_parameter(
    id: i32,
    parameter_name: &str,
    pool: &MySqlPool,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query!(
        r#"
            SELECT id.uuid_id
                FROM event_parameter p
                JOIN event_parameter_name n ON n.name = ?
                JOIN event_parameter_uuid id ON id.event_parameter_id = p.id
                WHERE p.name_id = n.id AND p.log_id = ?
        "#,
        parameter_name,
        id,
    )
    .fetch_all(pool)
    .await
    .map(|params| params.first().map(|param| param.uuid_id as i32))
}

pub async fn fetch_string_parameter(
    id: i32,
    parameter_name: &str,
    pool: &MySqlPool,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query!(
        r#"
            SELECT s.value
                FROM event_parameter p
                JOIN event_parameter_name n ON n.name = ?
                JOIN event_parameter_string s ON s.event_parameter_id = p.id
                WHERE p.name_id = n.id AND p.log_id = ?
        "#,
        parameter_name,
        id,
    )
    .fetch_all(pool)
    .await
    .map(|params| params.first().map(|param| param.value.to_string()))
}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event {id:?} can't be fetched because a field is missing.")]
    MissingField { id: i32 },
}
