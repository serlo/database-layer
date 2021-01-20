use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadState {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub thread_id: i32,
    pub archived: bool,
}

impl SetThreadState {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        let archived = abstract_event.name == "discussion/comment/archive";
        SetThreadState {
            thread_id: abstract_event.object_id,
            archived,
            abstract_event,
        }
    }
}
