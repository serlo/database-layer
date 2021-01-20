use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveEntityLink {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub child_id: i32,
    pub parent_id: i32,
}

impl RemoveEntityLink {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        RemoveEntityLink {
            __typename: "RemoveEntityLinkNotificationEvent".to_string(),
            child_id: abstract_event.object_id,
            // uses "parent" parameter
            parent_id: abstract_event.parameter_uuid_id,
            abstract_event,
        }
    }
}
