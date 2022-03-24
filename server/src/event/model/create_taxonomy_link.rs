use std::collections::HashMap;
use std::convert::TryFrom;

use crate::database::Executor;
use serde::Serialize;

use super::{AbstractEvent, Event, EventError, EventPayload, RawEventType};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyLinkEvent {
    entity_id: i32,
    entity_revision_id: i32,
}

impl TryFrom<&AbstractEvent> for CreateTaxonomyLinkEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let entity_id = abstract_event.uuid_parameters.try_get("repository")?;
        let entity_revision_id = abstract_event.object_id;

        Ok(CreateTaxonomyLinkEvent {
            entity_id,
            entity_revision_id,
        })
    }
}

pub struct CreateTaxonomyLinkEventPayload {
    raw_typename: RawEventType,
    child_id: i32,
    parent_id: i32,
    actor_id: i32,
    instance_id: i32,
}

impl CreateTaxonomyLinkEventPayload {
    pub fn new(child_id: i32, parent_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            raw_typename: RawEventType::CreateTaxonomyLink,
            child_id,
            parent_id,
            actor_id,
            instance_id,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let event = EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.parent_id,
            self.instance_id,
            HashMap::new(),
            [("object".to_string(), self.child_id)] //
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
