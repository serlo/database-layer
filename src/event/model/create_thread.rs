use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThread {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub object_id: i32,
    pub thread_id: i32,
}

impl CreateThread {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateThread {
            __typename: "CreateThreadNotificationEvent".to_string(),
            // uses "on" parameter
            object_id: abstract_event.parameter_uuid_id,
            thread_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
