use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

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
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Event> {
        let event_fut = sqlx::query!(
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

        let name = event_fut.name;
        let uuid_id = event_fut.uuid_id as i32;
        let parameter_uuid_id = event_fut
            .parameter_uuid_id
            .map(|id| id as i32)
            .unwrap_or(uuid_id);

        let e = AbstractEvent {
            __typename: name.parse::<EventType>()?,
            id: event_fut.id as i32,
            instance: event_fut.subdomain.to_string(),
            date: format_datetime(&event_fut.date),
            actor_id: event_fut.actor_id as i32,
            object_id: uuid_id,
            parameter_uuid_id,
            name,
        };

        let event = match e.name.as_str().parse()? {
            EventType::SetThreadState => Event::SetThreadState(SetThreadState::new(e)),
            EventType::CreateComment => Event::CreateComment(CreateComment::new(e)),
            EventType::CreateThread => Event::CreateThread(CreateThread::new(e)),
            EventType::CreateEntity => Event::CreateEntity(CreateEntity::new(e)),
            EventType::SetLicense => Event::SetLicense(SetLicense::new(e)),
            EventType::CreateEntityLink => Event::CreateEntityLink(CreateEntityLink::new(e)),
            EventType::RemoveEntityLink => Event::RemoveEntityLink(RemoveEntityLink::new(e)),
            EventType::CreateEntityRevision => {
                Event::CreateEntityRevision(CreateEntityRevision::new(e))
            }
            EventType::CheckoutRevision => {
                Event::CheckoutRevision(CheckoutRevision::new(e, pool).await)
            }
            EventType::RejectRevision => Event::RejectRevision(RejectRevision::new(e, pool).await),
            EventType::CreateTaxonomyLink => Event::CreateTaxonomyLink(CreateTaxonomyLink::new(e)),
            EventType::RemoveTaxonomyLink => Event::RemoveTaxonomyLink(RemoveTaxonomyLink::new(e)),
            EventType::CreateTaxonomyTerm => Event::CreateTaxonomyTerm(CreateTaxonomyTerm::new(e)),
            EventType::SetTaxonomyTerm => Event::SetTaxonomyTerm(SetTaxonomyTerm::new(e)),
            EventType::SetTaxonomyParent => {
                Event::SetTaxonomyParent(SetTaxonomyParent::new(e, pool).await)
            }
            EventType::SetUuidState => Event::SetUuidState(SetUuidState::new(e)),
            EventType::Unsupported => Event::Unsupported(Unsupported::new(e)),
        };

        Ok(event)
    }
}

pub async fn fetch_parameter_uuid_id(
    id: i32,
    parameter_name: &str,
    pool: &MySqlPool,
) -> Option<i32> {
    sqlx::query!(
        r#"
            SELECT id.uuid_id FROM event_parameter p
                JOIN event_parameter_name n ON n.name = ?
                JOIN event_parameter_uuid id ON id.event_parameter_id = p.id
                WHERE p.name_id = n.id AND p.log_id = ?
        "#,
        parameter_name,
        id,
    )
    .fetch_one(pool)
    .await
    .ok()
    .map(|o| o.uuid_id as i32)
}

pub async fn fetch_parameter_string(id: i32, parameter_name: &str, pool: &MySqlPool) -> String {
    sqlx::query!(
        r#"
            SELECT s.value FROM event_parameter p
                JOIN event_parameter_name n ON n.name = ?
                JOIN event_parameter_string s ON s.event_parameter_id = p.id
                WHERE p.name_id = n.id AND p.log_id = ?
        "#,
        parameter_name,
        id,
    )
    .fetch_one(pool)
    .await
    .ok()
    .map(|o| o.value)
    .unwrap_or_else(|| "".to_string())
}
