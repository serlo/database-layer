use crate::database::Executor;
use crate::event::{Event, EventError, EventPayload, RawEventType};
use serde::Serialize;
use std::collections::HashMap;

use super::abstract_event::AbstractEvent;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityEvent {
    entity_id: i32,
}

impl From<&AbstractEvent> for CreateEntityEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let entity_id = abstract_event.object_id;

        CreateEntityEvent { entity_id }
    }
}

pub struct CreateEntityEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    instance_id: i32,
    entity_id: i32,
}

impl CreateEntityEventPayload {
    pub fn new(entity_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            raw_typename: RawEventType::CreateEntity,
            actor_id,
            instance_id,
            entity_id,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        Ok(EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.entity_id,
            self.instance_id,
            HashMap::new(),
            HashMap::new(),
        )
        .save(executor)
        .await?)
    }
}
