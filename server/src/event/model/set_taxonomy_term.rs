use std::collections::HashMap;
use std::convert::TryFrom;

use crate::database::Executor;
use crate::event::{Event, EventPayload, RawEventType};
use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyTermEvent {
    id: i32,
}

impl TryFrom<&AbstractEvent> for SetTaxonomyTermEvent {
    type Error = EventError;
    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let id = abstract_event.id;

        Ok(SetTaxonomyTermEvent { id })
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

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let event = EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.taxonomy_term_id,
            self.instance_id,
            HashMap::new(),
            HashMap::new(),
        )
        .save(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(event)
    }
}
