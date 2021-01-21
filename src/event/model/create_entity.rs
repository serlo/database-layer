use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntity {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    entity_id: i32,
}

#[async_trait]
impl FromAbstractEvent for CreateEntity {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let entity_id = abstract_event.object_id;

        Ok(CreateEntity {
            abstract_event,

            entity_id,
        })
    }
}
