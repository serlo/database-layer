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
    pub fn new(abstract_event: AbstractEvent) -> Self {
        let trashed = abstract_event.name == "uuid/trash";
        SetUuidState {
            abstract_event,
            trashed,
        }
    }
}
