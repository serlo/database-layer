use anyhow::Result;
use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntity {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub entity_id: i32,
}

impl CreateEntity {
    pub async fn new(abstract_event: AbstractEvent) -> Result<Self> {
        let entity_id = abstract_event.object_id;

        Ok(CreateEntity {
            abstract_event,

            entity_id,
        })
    }
}
