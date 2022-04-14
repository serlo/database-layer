use std::collections::HashMap;
use std::convert::TryFrom;

use crate::database::Executor;
use crate::event::{Event, EventPayload, RawEventType};
use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyParentEvent {
    child_id: i32,
    previous_parent_id: Option<i32>,
    parent_id: Option<i32>,
}

impl TryFrom<&AbstractEvent> for SetTaxonomyParentEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let child_id = abstract_event.object_id;
        let previous_parent_id = abstract_event.uuid_parameters.get("from");
        let parent_id = abstract_event.uuid_parameters.get("to");

        Ok(SetTaxonomyParentEvent {
            child_id,
            previous_parent_id,
            parent_id,
        })
    }
}

pub struct SetTaxonomyParentEventPayload {
    raw_typename: RawEventType,
    child_id: i32,
    previous_parent_id: i32,
    parent_id: i32,
    actor_id: i32,
    instance_id: i32,
}

impl SetTaxonomyParentEventPayload {
    pub fn new(
        child_id: i32,
        previous_parent_id: i32,
        parent_id: i32,
        actor_id: i32,
        instance_id: i32,
    ) -> Self {
        Self {
            raw_typename: RawEventType::SetTaxonomyParent,
            child_id,
            previous_parent_id,
            parent_id,
            actor_id,
            instance_id,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let event = EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.child_id,
            self.instance_id,
            HashMap::new(),
            [
                ("from".to_string(), self.previous_parent_id),
                ("to".to_string(), self.parent_id),
            ]
            .iter()
            .cloned()
            .collect(),
        )
        .save(executor)
        .await?;

        Ok(event)
    }
}
