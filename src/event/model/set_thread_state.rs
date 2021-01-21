use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadState {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    thread_id: i32,
    archived: bool,
}

#[async_trait]
impl FromAbstractEvent for SetThreadState {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let thread_id = abstract_event.object_id;
        let archived = abstract_event.name == "discussion/comment/archive";

        Ok(SetThreadState {
            abstract_event,

            thread_id,
            archived,
        })
    }
}
