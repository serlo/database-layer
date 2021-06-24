use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use futures::stream::FuturesOrdered;
use futures::TryStreamExt;
use serde::Serialize;
use sqlx::MySqlPool;

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
use crate::notification::{Notifications, NotificationsError};

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

    pub async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let abstract_event = AbstractEvent::fetch_via_transaction(id, executor).await?;
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
        .execute(&mut transaction)
        .await?;

        let event_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
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
            .execute(&mut transaction)
            .await?;

            let parameter_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
                .fetch_one(&mut transaction)
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
            .execute(&mut transaction)
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
            .execute(&mut transaction)
            .await?;

            let parameter_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
                .fetch_one(&mut transaction)
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
            .execute(&mut transaction)
            .await?;
        }

        let event = Event::fetch_via_transaction(event_id, &mut transaction).await?;
        Notifications::create_notifications(&event, &mut transaction)
            .await
            .map_err(|error| match error {
                NotificationsError::DatabaseError { inner } => EventError::from(inner),
            })?;

        transaction.commit().await?;

        Ok(event)
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Events {
    events: Vec<Event>,
}

impl Events {
    pub async fn fetch(
        after: i32,
        max_events: i32,
        pool: &MySqlPool,
    ) -> Result<Events, sqlx::Error> {
        let event_ids = Events::fetch_event_ids(after, max_events, pool).await?;

        let mut event_tasks = FuturesOrdered::new();
        for event_id in event_ids {
            event_tasks.push(Events::fetch_event(event_id, pool));
        }

        let events: Vec<Option<Event>> = event_tasks.try_collect().await?;
        let events = events.into_iter().filter_map(|x| x).collect();
        Ok(Events { events })
    }

    pub async fn fetch_via_transaction<'a, E>(
        after: i32,
        max_events: i32,
        executor: E,
    ) -> Result<Events, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let event_ids = Events::fetch_event_ids(after, max_events, &mut transaction).await?;
        let mut events = Vec::new();

        for event_id in event_ids {
            if let Some(event) = Events::fetch_event(event_id, &mut transaction).await? {
                events.push(event);
            }
        }

        Ok(Events { events })
    }

    async fn fetch_event_ids<'a, E>(
        after: i32,
        max_events: i32,
        executor: E,
    ) -> Result<Vec<i32>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        Ok(sqlx::query!(
            "SELECT id FROM event_log WHERE id > ? ORDER BY id LIMIT ?",
            after,
            max_events
        )
        .fetch_all(executor)
        .await?
        .iter()
        .map(|record| record.id as i32)
        .collect())
    }

    async fn fetch_event<'a, E>(event_id: i32, executor: E) -> Result<Option<Event>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        match Event::fetch_via_transaction(event_id, executor).await {
            Ok(event) => Ok(Some(event)),
            // Ignore invalid events
            Err(EventError::InvalidInstance)
            | Err(EventError::MissingRequiredField)
            | Err(EventError::InvalidType)
            // Event has been deleted after fetching the ids
            | Err(EventError::NotFound) => Ok(None),
            Err(EventError::DatabaseError { inner }) => Err(inner),
        }
    }
}
