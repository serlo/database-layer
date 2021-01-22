use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntity {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    entity_id: i32,
}

impl From<AbstractEvent> for CreateEntity {
    fn from(abstract_event: AbstractEvent) -> Self {
        let entity_id = abstract_event.object_id;

        CreateEntity {
            abstract_event,

            entity_id,
        }
    }
}
