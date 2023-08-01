use std::collections::HashMap;

use crate::event::{Event, EventPayload, RawEventType};
use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyTermEvent {
    id: i32,
}

impl From<&AbstractEvent> for SetTaxonomyTermEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        SetTaxonomyTermEvent {
            id: abstract_event.id,
        }
    }
}

pub struct SetTaxonomyTermEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    taxonomy_term_id: i32,
    instance_id: i32,
}

impl SetTaxonomyTermEventPayload {
    pub fn new(taxonomy_term_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            raw_typename: RawEventType::SetTaxonomyTerm,
            actor_id,
            taxonomy_term_id,
            instance_id,
        }
    }

    pub async fn save<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> Result<Event, EventError> {
        EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.taxonomy_term_id,
            self.instance_id,
            HashMap::new(),
            HashMap::new(),
        )
        .save(acquire_from)
        .await
    }
}
