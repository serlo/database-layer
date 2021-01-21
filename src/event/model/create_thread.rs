use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThread {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    object_id: i32,
    thread_id: i32,
}

#[async_trait]
impl FromAbstractEvent for CreateThread {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        // uses "on" parameter
        let object_id = abstract_event.parameter_uuid_id;
        let thread_id = abstract_event.object_id;

        Ok(CreateThread {
            abstract_event,

            object_id,
            thread_id,
        })
    }
}
