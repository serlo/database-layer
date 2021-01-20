use anyhow::Result;
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
    pub async fn new(abstract_event: AbstractEvent) -> Result<Self> {
        let thread_id = abstract_event.object_id;
        let archived = abstract_event.name == "discussion/comment/archive";

        Ok(SetThreadState {
            abstract_event,

            thread_id,
            archived,
        })
    }
}
