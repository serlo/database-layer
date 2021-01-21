use anyhow::Result;
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
    pub async fn fetch(abstract_event: AbstractEvent) -> Result<Self> {
        // uses "discussion" parameter
        let thread_id = abstract_event.parameter_uuid_id;
        let comment_id = abstract_event.object_id;

        Ok(CreateComment {
            abstract_event,

            thread_id,
            comment_id,
        })
    }
}
