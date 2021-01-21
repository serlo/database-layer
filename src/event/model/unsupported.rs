use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unsupported {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    r#type: String,
    error: String,
}

#[async_trait]
impl FromAbstractEvent for Unsupported {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let r#type = abstract_event.name.to_string();

        Ok(Unsupported {
            abstract_event,

            r#type,
            error: "unsupported".to_string(),
        })
    }
}
