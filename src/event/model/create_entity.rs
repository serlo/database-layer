use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntity {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub entity_id: i32,
}

impl CreateEntity {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateEntity {
            entity_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
