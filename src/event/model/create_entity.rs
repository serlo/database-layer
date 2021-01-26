use serde::Serialize;

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
