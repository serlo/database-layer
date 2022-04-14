use std::collections::HashMap;
use std::convert::TryFrom;

use crate::database::Executor;
use serde::Serialize;

use super::{AbstractEvent, Event, EventError, EventPayload, RawEventType};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyLinkEvent {
    entity_id: i32,
    taxonomy_term_id: i32,
}

impl TryFrom<&AbstractEvent> for CreateTaxonomyLinkEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let entity_id = abstract_event.uuid_parameters.try_get("object")?;
        let taxonomy_term_id = abstract_event.object_id;

        Ok(CreateTaxonomyLinkEvent {
            entity_id,
            taxonomy_term_id,
        })
    }
}

pub struct CreateTaxonomyLinkEventPayload {
    raw_typename: RawEventType,
    entity_id: i32,
    taxonomy_term_id: i32,
    actor_id: i32,
    instance_id: i32,
}

impl CreateTaxonomyLinkEventPayload {
    pub fn new(entity_id: i32, taxonomy_term_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            raw_typename: RawEventType::CreateTaxonomyLink,
            entity_id,
            taxonomy_term_id,
            actor_id,
            instance_id,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.taxonomy_term_id,
            self.instance_id,
            HashMap::new(),
            [("object".to_string(), self.entity_id)]
                .iter()
                .cloned()
                .collect(),
        )
        .save(executor)
        .await
    }
}
