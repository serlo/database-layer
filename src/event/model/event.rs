use serde::Serialize;
use sqlx::MySqlPool;

use crate::format_datetime;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::checkout_revision::CheckoutRevision;
use super::create_comment::CreateComment;
use super::create_entity::CreateEntity;
use super::create_entity_link::CreateEntityLink;
use super::create_entity_revision::CreateEntityRevision;
use super::create_taxonomy_link::CreateTaxonomyLink;
use super::create_taxonomy_term::CreateTaxonomyTerm;
use super::create_thread::CreateThread;
use super::event_type::EventType;
use super::reject_revision::RejectRevision;
use super::remove_entity_link::RemoveEntityLink;
use super::remove_taxonomy_link::RemoveTaxonomyLink;
use super::set_license::SetLicense;
use super::set_taxonomy_parent::SetTaxonomyParent;
use super::set_taxonomy_term::SetTaxonomyTerm;
use super::set_thread_state::SetThreadState;
use super::set_uuid_state::SetUuidState;
use super::unsupported::Unsupported;
use super::EventError;

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

impl Event {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Event, EventError> {
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
            .await
            .map_err(|error| {
                match error {
                    sqlx::Error::RowNotFound => EventError::NotFound,
                    inner => EventError::DatabaseError { inner },
                }
            })?;

        let name = event.name;
        let uuid_id = event.uuid_id as i32;
        let parameter_uuid_id = event
            .parameter_uuid_id
            .map(|id| id as i32)
            .unwrap_or(uuid_id);

        let abstract_event = AbstractEvent {
            __typename: name.parse().map_err(|_error| EventError::InvalidType)?,
            id: event.id as i32,
            instance: event.subdomain.to_string(),
            date: format_datetime(&event.date),
            actor_id: event.actor_id as i32,
            object_id: uuid_id,
            parameter_uuid_id,
            name,
        };

        let event = match abstract_event.__typename {
            EventType::CheckoutRevision => {
                Event::CheckoutRevision(CheckoutRevision::fetch(abstract_event, pool).await?)
            }
            EventType::CreateComment => {
                Event::CreateComment(CreateComment::fetch(abstract_event, pool).await?)
            }
            EventType::CreateEntity => {
                Event::CreateEntity(CreateEntity::fetch(abstract_event, pool).await?)
            }
            EventType::CreateEntityLink => {
                Event::CreateEntityLink(CreateEntityLink::fetch(abstract_event, pool).await?)
            }
            EventType::CreateEntityRevision => Event::CreateEntityRevision(
                CreateEntityRevision::fetch(abstract_event, pool).await?,
            ),
            EventType::CreateTaxonomyLink => {
                Event::CreateTaxonomyLink(CreateTaxonomyLink::fetch(abstract_event, pool).await?)
            }
            EventType::CreateTaxonomyTerm => {
                Event::CreateTaxonomyTerm(CreateTaxonomyTerm::fetch(abstract_event, pool).await?)
            }
            EventType::CreateThread => {
                Event::CreateThread(CreateThread::fetch(abstract_event, pool).await?)
            }
            EventType::RejectRevision => {
                Event::RejectRevision(RejectRevision::fetch(abstract_event, pool).await?)
            }
            EventType::RemoveEntityLink => {
                Event::RemoveEntityLink(RemoveEntityLink::fetch(abstract_event, pool).await?)
            }
            EventType::RemoveTaxonomyLink => {
                Event::RemoveTaxonomyLink(RemoveTaxonomyLink::fetch(abstract_event, pool).await?)
            }
            EventType::SetLicense => {
                Event::SetLicense(SetLicense::fetch(abstract_event, pool).await?)
            }
            EventType::SetTaxonomyParent => {
                Event::SetTaxonomyParent(SetTaxonomyParent::fetch(abstract_event, pool).await?)
            }
            EventType::SetTaxonomyTerm => {
                Event::SetTaxonomyTerm(SetTaxonomyTerm::fetch(abstract_event, pool).await?)
            }
            EventType::SetThreadState => {
                Event::SetThreadState(SetThreadState::fetch(abstract_event, pool).await?)
            }
            EventType::SetUuidState => {
                Event::SetUuidState(SetUuidState::fetch(abstract_event, pool).await?)
            }
            EventType::Unsupported => {
                Event::Unsupported(Unsupported::fetch(abstract_event, pool).await?)
            }
        };

        Ok(event)
    }

    pub async fn fetch_uuid_parameter(
        id: i32,
        parameter_name: &str,
        pool: &MySqlPool,
    ) -> Result<Option<i32>, EventError> {
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
        .map_err(|inner| EventError::DatabaseError { inner })
    }

    pub async fn fetch_string_parameter(
        id: i32,
        parameter_name: &str,
        pool: &MySqlPool,
    ) -> Result<Option<String>, EventError> {
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
        .map_err(|inner| EventError::DatabaseError { inner })
    }
}
