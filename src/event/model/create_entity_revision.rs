use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityRevision {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    entity_id: i32,
    entity_revision_id: i32,
}

#[async_trait]
impl FromAbstractEvent for CreateEntityRevision {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        // uses "repository" parameter
        let entity_id = abstract_event.parameter_uuid_id;
        let entity_revision_id = abstract_event.object_id;

        Ok(CreateEntityRevision {
            abstract_event,

            entity_id,
            entity_revision_id,
        })
    }
}
