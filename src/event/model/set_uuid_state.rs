use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    trashed: bool,
}

#[async_trait]
impl FromAbstractEvent for SetUuidState {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let trashed = abstract_event.name == "uuid/trash";

        Ok(SetUuidState {
            abstract_event,

            trashed,
        })
    }
}
