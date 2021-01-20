use anyhow::Result;
use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub trashed: bool,
}

impl SetUuidState {
    pub async fn new(abstract_event: AbstractEvent) -> Result<Self> {
        let trashed = abstract_event.name == "uuid/trash";

        Ok(SetUuidState {
            abstract_event,

            trashed,
        })
    }
}
