use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntity {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub entity_id: i32,
}

impl CreateEntity {
    pub fn new(abstract_event: AbstractEvent) -> CreateEntity {
        CreateEntity {
            __typename: "CreateEntityNotificationEvent".to_string(),
            entity_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
