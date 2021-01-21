use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityLink {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    child_id: i32,
    parent_id: i32,
}

#[async_trait]
impl FromAbstractEvent for CreateEntityLink {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let child_id = abstract_event.object_id;
        // uses "parent" parameter
        let parent_id = abstract_event.parameter_uuid_id;

        Ok(CreateEntityLink {
            abstract_event,

            child_id,
            parent_id,
        })
    }
}
