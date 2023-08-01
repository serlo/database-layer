use crate::event::{Event, EventError, EventPayload};
use serde::Serialize;
use std::collections::HashMap;

use super::abstract_event::AbstractEvent;
use super::RawEventType;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLicenseEvent {
    repository_id: i32,
}

impl From<&AbstractEvent> for SetLicenseEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let repository_id = abstract_event.object_id;

        SetLicenseEvent { repository_id }
    }
}

pub struct CreateSetLicenseEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    instance_id: i32,
    entity_id: i32,
}

impl CreateSetLicenseEventPayload {
    pub fn new(entity_id: i32, actor_id: i32, instance_id: i32) -> Self {
        Self {
            raw_typename: RawEventType::SetLicense,
            actor_id,
            instance_id,
            entity_id,
        }
    }

    pub async fn save<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> Result<Event, EventError> {
        EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.entity_id,
            self.instance_id,
            HashMap::new(),
            HashMap::new(),
        )
        .save(acquire_from)
        .await
    }
}
