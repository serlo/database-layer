use crate::database::Executor;
use crate::event::{Event, EventError, EventPayload, RawEventType};
use serde::Serialize;
use std::collections::HashMap;

use super::abstract_event::AbstractEvent;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyTermEvent {
    taxonomy_term_id: i32,
}

impl From<&AbstractEvent> for CreateTaxonomyTermEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let taxonomy_term_id = abstract_event.object_id;

        CreateTaxonomyTermEvent { taxonomy_term_id }
    }
}

pub struct CreateTaxonomyTermEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    instance_id: i32,
    taxonomy_term_id: i32,
}

impl CreateTaxonomyTermEventPayload {
    pub fn new(taxonomy_term_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            raw_typename: RawEventType::CreateTaxonomyTerm,
            actor_id,
            instance_id,
            taxonomy_term_id,
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
            HashMap::new(),
        )
        .save(executor)
        .await
    }
}
