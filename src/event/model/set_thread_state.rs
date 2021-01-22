use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadState {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    thread_id: i32,
    archived: bool,
}

impl From<AbstractEvent> for SetThreadState {
    fn from(abstract_event: AbstractEvent) -> Self {
        let thread_id = abstract_event.object_id;
        let archived = abstract_event.raw_typename == "discussion/comment/archive";

        SetThreadState {
            abstract_event,

            thread_id,
            archived,
        }
    }
}
