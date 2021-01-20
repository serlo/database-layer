use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub trashed: bool,
}

impl SetUuidState {
    pub fn new(abstract_event: AbstractEvent, trashed: bool) -> Self {
        SetUuidState {
            abstract_event,
            trashed,
        }
    }
}
