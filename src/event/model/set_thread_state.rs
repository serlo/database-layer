use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadState {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub thread_id: i32,
    pub archived: bool,
}

impl SetThreadState {
    pub fn new(abstract_event: AbstractEvent, archived: bool) -> Self {
        SetThreadState {
            __typename: "SetThreadStateNotificationEvent".to_string(),
            thread_id: abstract_event.object_id,
            archived,
            abstract_event,
        }
    }
}
