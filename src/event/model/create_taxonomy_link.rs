use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyLink {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    parent_id: i32,
    child_id: i32,
}

#[async_trait]
impl FromAbstractEvent for CreateTaxonomyLink {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let parent_id = abstract_event.object_id;
        // uses "object" parameter
        let child_id = abstract_event.parameter_uuid_id;

        Ok(CreateTaxonomyLink {
            abstract_event,

            parent_id,
            child_id,
        })
    }
}
