use std::collections::HashMap;
use std::convert::TryFrom;

use serde::Serialize;

use super::{AbstractEvent, Event, EventError, EventPayload, RawEventType};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityRevisionEvent {
    entity_id: i32,
    entity_revision_id: i32,
}

impl TryFrom<&AbstractEvent> for CreateEntityRevisionEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let entity_id = abstract_event.uuid_parameters.try_get("repository")?;
        let entity_revision_id = abstract_event.object_id;

        Ok(CreateEntityRevisionEvent {
            entity_id,
            entity_revision_id,
        })
    }
}

pub struct CreateEntityRevisionEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    instance_id: i32,
    entity_id: i32,
    entity_revision_id: i32,
}

impl CreateEntityRevisionEventPayload {
    pub fn new(entity_id: i32, entity_revision_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            raw_typename: RawEventType::CreateEntityRevision,
            actor_id,
            instance_id,
            entity_id,
            entity_revision_id,
        }
    }

    pub async fn save<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> Result<Event, EventError> {
        EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.entity_revision_id,
            self.instance_id,
            HashMap::new(),
            [("repository".to_string(), self.entity_id)]
                .iter()
                .cloned()
                .collect(),
        )
        .save(acquire_from)
        .await
    }
}
