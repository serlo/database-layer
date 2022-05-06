use std::collections::HashMap;
use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::{Event, EventError, EventPayload, RawEventType};
use crate::database::Executor;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityLinkEvent {
    child_id: i32,
    parent_id: i32,
}

impl TryFrom<&AbstractEvent> for EntityLinkEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let child_id = abstract_event.object_id;
        let parent_id = abstract_event.uuid_parameters.try_get("parent")?;

        Ok(EntityLinkEvent {
            child_id,
            parent_id,
        })
    }
}

pub struct EntityLinkEventPayload {
    child_id: i32,
    actor_id: i32,
    parent_id: i32,
    instance_id: i32,
}

impl EntityLinkEventPayload {
    pub fn new(child_id: i32, parent_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            child_id,
            actor_id,
            parent_id,
            instance_id,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let event = EventPayload::new(
            RawEventType::CreateEntityLink,
            self.actor_id,
            self.child_id,
            self.instance_id,
            HashMap::new(),
            [("parent".to_string(), self.parent_id)]
                .iter()
                .cloned()
                .collect(),
        )
        .save(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(event)
    }
}
