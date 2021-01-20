use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadState {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub thread_id: i32,
    pub archived: bool,
}

impl SetThreadState {
    pub fn new(abstract_event: AbstractEvent, archived: bool) -> Self {
        SetThreadState {
            thread_id: abstract_event.object_id,
            archived,
            abstract_event,
        }
    }
}
