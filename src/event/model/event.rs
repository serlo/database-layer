use std::convert::TryInto;

use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, EventStringParameters, EventUuidParameters};
use super::create_comment::CreateComment;
use super::create_entity::CreateEntity;
use super::create_entity_revision::CreateEntityRevision;
use super::create_thread::CreateThread;
use super::entity_link::EntityLink;
use super::event_type::EventType;
use super::revision::Revision;
use super::set_license::SetLicense;
use super::set_taxonomy_parent::SetTaxonomyParent;
use super::set_thread_state::SetThreadState;
use super::set_uuid_state::SetUuidState;
use super::taxonomy_link::TaxonomyLink;
use super::taxonomy_term::TaxonomyTerm;
use super::unsupported::Unsupported;
use super::EventError;

#[derive(Serialize)]
pub struct Event {
    #[serde(flatten)]
    abstract_event: AbstractEvent,
    #[serde(flatten)]
    concrete_event: ConcreteEvent,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ConcreteEvent {
    SetThreadState(SetThreadState),
    CreateComment(CreateComment),
    CreateThread(CreateThread),
    CreateEntity(CreateEntity),
    SetLicense(SetLicense),
    CreateEntityLink(EntityLink),
    RemoveEntityLink(EntityLink),
    CreateEntityRevision(CreateEntityRevision),
    CheckoutRevision(Revision),
    RejectRevision(Revision),
    CreateTaxonomyLink(TaxonomyLink),
    RemoveTaxonomyLink(TaxonomyLink),
    CreateTaxonomyTerm(TaxonomyTerm),
    SetTaxonomyTerm(TaxonomyTerm),
    SetTaxonomyParent(SetTaxonomyParent),
    SetUuidState(SetUuidState),
    Unsupported(Unsupported),
}

impl Event {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Event, EventError> {
        let event = sqlx::query!(
            r#"
                SELECT l.id, l.actor_id, l.uuid_id, l.date, i.subdomain, e.name
                    FROM event_log l
                    LEFT JOIN event_parameter p ON l.id = p.log_id
                    JOIN instance i ON l.instance_id = i.id
                    JOIN event e ON l.event_id = e.id
                    WHERE l.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => EventError::NotFound,
            inner => EventError::DatabaseError { inner },
        })?;

        let string_parameters = sqlx::query!(
            r#"
                SELECT n.name, s.value
                    FROM event_parameter p
                    JOIN event_parameter_name n ON n.id = p.name_id
                    JOIN event_parameter_string s ON s.event_parameter_id = p.id
                    WHERE p.name_id = n.id AND p.log_id = ?
            "#,
            id
        )
        .fetch_all(pool);

        let uuid_parameters = sqlx::query!(
            r#"
                SELECT n.name, u.uuid_id
                    FROM event_parameter p
                    JOIN event_parameter_name n ON n.id = p.name_id
                    JOIN event_parameter_uuid u ON u.event_parameter_id = p.id
                    WHERE p.name_id = n.id AND p.log_id = ?
            "#,
            id
        )
        .fetch_all(pool);

        let (string_parameters, uuid_parameters) = try_join!(string_parameters, uuid_parameters)
            .map_err(|inner| EventError::DatabaseError { inner })?;

        let raw_typename = event.name;
        let uuid_id = event.uuid_id as i32;

        let string_parameters = string_parameters
            .into_iter()
            .map(|param| (param.name, param.value))
            .collect();
        let string_parameters = EventStringParameters(string_parameters);

        let uuid_parameters = uuid_parameters
            .into_iter()
            .map(|param| (param.name, param.uuid_id as i32))
            .collect();
        let uuid_parameters = EventUuidParameters(uuid_parameters);

        let abstract_event = AbstractEvent {
            __typename: raw_typename
                .parse()
                .map_err(|_error| EventError::InvalidType)?,
            id: event.id as i32,
            instance: event.subdomain.to_string(),
            date: event.date.into(),
            actor_id: event.actor_id as i32,
            object_id: uuid_id,
            raw_typename,

            string_parameters,
            uuid_parameters,
        };

        let abstract_event_ref = &abstract_event;

        let concrete_event = match abstract_event_ref.__typename {
            EventType::CheckoutRevision => {
                ConcreteEvent::CheckoutRevision(abstract_event_ref.try_into()?)
            }
            EventType::CreateComment => {
                ConcreteEvent::CreateComment(abstract_event_ref.try_into()?)
            }
            EventType::CreateEntity => ConcreteEvent::CreateEntity(abstract_event_ref.into()),
            EventType::CreateEntityLink => {
                ConcreteEvent::CreateEntityLink(abstract_event_ref.try_into()?)
            }
            EventType::CreateEntityRevision => {
                ConcreteEvent::CreateEntityRevision(abstract_event_ref.try_into()?)
            }
            EventType::CreateTaxonomyLink => {
                ConcreteEvent::CreateTaxonomyLink(abstract_event_ref.try_into()?)
            }
            EventType::CreateTaxonomyTerm => {
                ConcreteEvent::CreateTaxonomyTerm(abstract_event_ref.into())
            }
            EventType::CreateThread => ConcreteEvent::CreateThread(abstract_event_ref.try_into()?),
            EventType::RejectRevision => {
                ConcreteEvent::RejectRevision(abstract_event_ref.try_into()?)
            }
            EventType::RemoveEntityLink => {
                ConcreteEvent::RemoveEntityLink(abstract_event_ref.try_into()?)
            }
            EventType::RemoveTaxonomyLink => {
                ConcreteEvent::RemoveTaxonomyLink(abstract_event_ref.try_into()?)
            }
            EventType::SetLicense => ConcreteEvent::SetLicense(abstract_event_ref.into()),
            EventType::SetTaxonomyParent => {
                ConcreteEvent::SetTaxonomyParent(abstract_event_ref.try_into()?)
            }
            EventType::SetTaxonomyTerm => ConcreteEvent::SetTaxonomyTerm(abstract_event_ref.into()),
            EventType::SetThreadState => ConcreteEvent::SetThreadState(abstract_event_ref.into()),
            EventType::SetUuidState => ConcreteEvent::SetUuidState(abstract_event_ref.into()),
            EventType::Unsupported => ConcreteEvent::Unsupported(abstract_event_ref.into()),
        };

        Ok(Event {
            abstract_event,
            concrete_event,
        })
    }
}
