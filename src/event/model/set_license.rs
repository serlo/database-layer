use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLicense {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    repository_id: i32,
}

#[async_trait]
impl FromAbstractEvent for SetLicense {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let repository_id = abstract_event.object_id;

        Ok(SetLicense {
            abstract_event,

            repository_id,
        })
    }
}
