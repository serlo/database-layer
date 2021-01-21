use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComment {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    thread_id: i32,
    comment_id: i32,
}

#[async_trait]
impl FromAbstractEvent for CreateComment {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
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
