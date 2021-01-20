use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityLink {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub child_id: i32,
    pub parent_id: i32,
}

impl CreateEntityLink {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateEntityLink {
            __typename: "CreateEntityLinkNotificationEvent".to_string(),
            child_id: abstract_event.object_id,
            // uses "parent" parameter
            parent_id: abstract_event.parameter_uuid_id,
            abstract_event,
        }
    }
}
