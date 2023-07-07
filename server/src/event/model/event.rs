use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use futures::TryStreamExt;
use serde::Serialize;
use sqlx::MySqlPool;

use super::super::messages::*;
use super::abstract_event::AbstractEvent;
use super::create_comment::CreateCommentEvent;
use super::create_entity::CreateEntityEvent;
use super::create_entity_revision::CreateEntityRevisionEvent;
use super::create_thread::CreateThreadEvent;
use super::entity_link::EntityLinkEvent;
use super::event_type::{EventType, RawEventType};
use super::revision::RevisionEvent;
use super::set_license::SetLicenseEvent;
use super::set_taxonomy_parent::SetTaxonomyParentEvent;
use super::set_thread_state::SetThreadStateEvent;
use super::set_uuid_state::SetUuidStateEvent;
use super::taxonomy_link::TaxonomyLinkEvent;
use super::taxonomy_term::TaxonomyTermEvent;
use super::EventError;
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::event::{EventStringParameters, EventUuidParameters};
use crate::instance::Instance;
use crate::notification::Notifications;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Event {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    #[serde(flatten)]
    pub concrete_event: ConcreteEvent,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ConcreteEvent {
    SetThreadState(SetThreadStateEvent),
    CreateComment(CreateCommentEvent),
    CreateThread(CreateThreadEvent),
    CreateEntity(CreateEntityEvent),
    SetLicense(SetLicenseEvent),
    CreateEntityLink(EntityLinkEvent),
    RemoveEntityLink(EntityLinkEvent),
    CreateEntityRevision(CreateEntityRevisionEvent),
    CheckoutRevision(RevisionEvent),
    RejectRevision(RevisionEvent),
    CreateTaxonomyLink(TaxonomyLinkEvent),
    RemoveTaxonomyLink(TaxonomyLinkEvent),
    CreateTaxonomyTerm(TaxonomyTermEvent),
    SetTaxonomyTerm(TaxonomyTermEvent),
    SetTaxonomyParent(SetTaxonomyParentEvent),
    SetUuidState(SetUuidStateEvent),
}

impl Event {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Event, EventError> {
        let abstract_event = AbstractEvent::fetch(id, pool).await?;
        abstract_event.try_into()
    }

    pub async fn fetch_via_transaction<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        id: i32,
        acquire_from: A,
    ) -> Result<Event, EventError> {
        let abstract_event = AbstractEvent::fetch_via_transaction(id, acquire_from).await?;
        abstract_event.try_into()
    }
}

impl TryFrom<AbstractEvent> for Event {
    type Error = EventError;

    fn try_from(abstract_event: AbstractEvent) -> Result<Self, Self::Error> {
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
        };

        Ok(Self {
            abstract_event,
            concrete_event,
        })
    }
}

pub struct EventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    object_id: i32,
    instance_id: i32,
    date: DateTime,

    string_parameters: HashMap<String, String>,
    uuid_parameters: HashMap<String, i32>,
}

impl EventPayload {
    pub fn new(
        raw_typename: RawEventType,
        actor_id: i32,
        object_id: i32,
        instance_id: i32,
        string_parameters: HashMap<String, String>,
        uuid_parameters: HashMap<String, i32>,
    ) -> Self {
        Self {
            raw_typename,
            actor_id,
            object_id,
            instance_id,
            date: DateTime::now(),

            string_parameters,
            uuid_parameters,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                SELECT *
                FROM user
                WHERE id = ?
            "#,
            self.actor_id,
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(EventError::MissingUser)?;

        sqlx::query!(
            r#"
                INSERT INTO event_log (actor_id, event_id, uuid_id, instance_id, date)
                    SELECT ?, id, ?, ?, ?
                    FROM event
                    WHERE name = ?
            "#,
            self.actor_id,
            self.object_id,
            self.instance_id,
            self.date,
            self.raw_typename,
        )
        .execute(&mut *transaction)
        .await?;

        let event_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut *transaction)
            .await?
            .id as i32;

        for (parameter, value) in &self.string_parameters {
            sqlx::query!(
                r#"
                    INSERT INTO event_parameter (log_id, name_id)
                        SELECT ?, id
                        FROM event_parameter_name
                        WHERE name = ?
                "#,
                event_id,
                parameter
            )
            .execute(&mut *transaction)
            .await?;

            let parameter_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
                .fetch_one(&mut *transaction)
                .await?
                .id as i32;

            sqlx::query!(
                r#"
                    INSERT INTO event_parameter_string (value, event_parameter_id)
                        VALUES (?, ?)
                "#,
                value,
                parameter_id
            )
            .execute(&mut *transaction)
            .await?;
        }

        for (parameter, uuid_id) in &self.uuid_parameters {
            sqlx::query!(
                r#"
                    INSERT INTO event_parameter (log_id, name_id)
                        SELECT ?, id
                        FROM event_parameter_name
                        WHERE name = ?
                "#,
                event_id,
                parameter
            )
            .execute(&mut *transaction)
            .await?;

            let parameter_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
                .fetch_one(&mut *transaction)
                .await?
                .id as i32;

            sqlx::query!(
                r#"
                    INSERT INTO event_parameter_uuid (uuid_id, event_parameter_id)
                        VALUES (?, ?)
                "#,
                uuid_id,
                parameter_id
            )
            .execute(&mut *transaction)
            .await?;
        }

        let event = Event::fetch_via_transaction(event_id, &mut *transaction).await?;
        Notifications::create_notifications(&event, &mut *transaction).await?;

        transaction.commit().await?;

        Ok(event)
    }
}

impl Event {
    pub async fn fetch_events<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &events_query::Payload,
        acquire_from: A,
    ) -> Result<events_query::Output, sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;
        let instance_id = match payload.instance.as_ref() {
            Some(instance) => Some(Instance::fetch_id(instance, &mut *transaction).await?),
            None => None,
        };
        let mut event_records = sqlx::query!(
            r#"
                SELECT
                    el.id,
                    i.subdomain                           AS instance,
                    e.name                                AS raw_typename,
                    el.actor_id,
                    el.date,
                    el.uuid_id                            AS object_id,
                    JSON_REMOVE(
                        JSON_OBJECTAGG(
                            CASE WHEN epn.name IS NOT NULL THEN epn.name ELSE "__unused_key" END,
                            eps.value
                        ),
                        "$.__unused_key"
                    )   AS string_parameters,
                    JSON_REMOVE(
                        JSON_OBJECTAGG(
                            CASE WHEN epn.name IS NOT NULL THEN epn.name ELSE "__unused_key" END,
                            epu.uuid_id
                        ),
                        "$.__unused_key"
                    ) AS uuid_parameters
                FROM event_log el
                    JOIN event e ON e.id = el.event_id
                    JOIN instance i on i.id = el.instance_id
                    LEFT JOIN uuid u ON u.id = el.uuid_id
                    LEFT JOIN event_parameter ep ON ep.log_id = el.id
                    LEFT JOIN event_parameter_name epn ON epn.id = ep.name_id
                    LEFT JOIN event_parameter_string eps ON eps.event_parameter_id = ep.id
                    LEFT JOIN event_parameter_uuid epu ON epu.event_parameter_id = ep.id
                WHERE
                  NOT (e.name = "entity/revision/checkout" AND u.discriminator = "pageRevision")
                  AND (? IS NULL OR el.id < ?)
                  AND (? IS NULL OR el.actor_id = ?)
                  AND (? IS NULL OR el.uuid_id = ? OR epu.uuid_id = ?)
                  AND (? IS NULL OR el.instance_id = ?)
                GROUP BY el.id
                ORDER BY el.id DESC
                LIMIT ?
            "#,
            payload.after,
            payload.after,
            payload.actor_id,
            payload.actor_id,
            payload.object_id,
            payload.object_id,
            payload.object_id,
            instance_id,
            instance_id,
            payload.first + 1
        )
        .fetch(&mut *transaction);

        let mut events: Vec<Event> = Vec::new();
        let mut has_next_page = false;

        while let Some(record) = event_records.try_next().await? {
            if events.len() as i32 == payload.first {
                has_next_page = true;
                break;
            }
            let instance = match record.instance.parse() {
                Ok(instance) => instance,
                _ => continue,
            };

            let raw_typename: RawEventType = match record.raw_typename.parse() {
                Ok(typename) => typename,
                _ => continue,
            };

            let string_parameters = record
                .string_parameters
                .and_then(|value| {
                    value.as_object().map(|object| {
                        object
                            .iter()
                            .filter_map(|(key, value)| {
                                value.as_str().map(|value| (key.clone(), value.to_string()))
                            })
                            .collect()
                    })
                })
                .unwrap_or_default();

            let uuid_parameters = record
                .uuid_parameters
                .and_then(|value| {
                    value.as_object().map(|object| {
                        object
                            .iter()
                            .filter_map(|(key, value)| {
                                value.as_i64().map(|value| (key.clone(), value as i32))
                            })
                            .collect()
                    })
                })
                .unwrap_or_default();

            let abstract_event = AbstractEvent {
                __typename: raw_typename.clone().into(),
                id: record.id as i32,
                instance,
                actor_id: record.actor_id as i32,
                object_id: record.object_id as i32,
                date: record.date.into(),
                raw_typename,
                string_parameters: EventStringParameters(string_parameters),
                uuid_parameters: EventUuidParameters(uuid_parameters),
            };

            if let Ok(event) = abstract_event.try_into() {
                events.push(event);
            }
        }

        Ok(events_query::Output {
            events,
            has_next_page,
        })
    }
}
