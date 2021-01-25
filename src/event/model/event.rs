use std::convert::TryInto;

use futures::try_join;
use serde::Serialize;

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
use crate::database::Executor;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Event {
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
    pub async fn fetch<'a, E>(id: i32, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor
            .begin()
            .await
            .map_err(|inner| EventError::DatabaseError { inner })?;
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
        .fetch_one(&mut transaction)
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
        .fetch_all(&mut transaction)
        .await
        .map_err(|inner| EventError::DatabaseError { inner })?;

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
        .fetch_all(&mut transaction)
        .await
        .map_err(|inner| EventError::DatabaseError { inner })?;

        // let (string_parameters, uuid_parameters) = try_join!(string_parameters, uuid_parameters)
        //     .map_err(|inner| EventError::DatabaseError { inner })?;

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

        let event = match abstract_event.__typename {
            EventType::CheckoutRevision => Event::CheckoutRevision(abstract_event.try_into()?),
            EventType::CreateComment => Event::CreateComment(abstract_event.try_into()?),
            EventType::CreateEntity => Event::CreateEntity(abstract_event.into()),
            EventType::CreateEntityLink => Event::CreateEntityLink(abstract_event.try_into()?),
            EventType::CreateEntityRevision => {
                Event::CreateEntityRevision(abstract_event.try_into()?)
            }
            EventType::CreateTaxonomyLink => Event::CreateTaxonomyLink(abstract_event.try_into()?),
            EventType::CreateTaxonomyTerm => Event::CreateTaxonomyTerm(abstract_event.into()),
            EventType::CreateThread => Event::CreateThread(abstract_event.try_into()?),
            EventType::RejectRevision => Event::RejectRevision(abstract_event.try_into()?),
            EventType::RemoveEntityLink => Event::RemoveEntityLink(abstract_event.try_into()?),
            EventType::RemoveTaxonomyLink => Event::RemoveTaxonomyLink(abstract_event.try_into()?),
            EventType::SetLicense => Event::SetLicense(abstract_event.into()),
            EventType::SetTaxonomyParent => Event::SetTaxonomyParent(abstract_event.try_into()?),
            EventType::SetTaxonomyTerm => Event::SetTaxonomyTerm(abstract_event.into()),
            EventType::SetThreadState => Event::SetThreadState(abstract_event.into()),
            EventType::SetUuidState => Event::SetUuidState(abstract_event.into()),
            EventType::Unsupported => Event::Unsupported(abstract_event.into()),
        };

        Ok(event)
    }
}
