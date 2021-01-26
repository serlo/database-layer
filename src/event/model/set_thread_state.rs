use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadStateEvent {
    thread_id: i32,
    archived: bool,
}

impl From<&AbstractEvent> for SetThreadStateEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let thread_id = abstract_event.object_id;
        let archived = abstract_event.raw_typename == "discussion/comment/archive";

        SetThreadStateEvent {
            thread_id,
            archived,
        }
    }
}
