use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComment {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub thread_id: i32,
    pub comment_id: i32,
}

impl CreateComment {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateComment {
            // uses "discussion" parameter
            thread_id: abstract_event.parameter_uuid_id,
            comment_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
