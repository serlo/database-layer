use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub trashed: bool,
}

impl SetUuidState {
    pub fn new(abstract_event: AbstractEvent, trashed: bool) -> Self {
        SetUuidState {
            __typename: "SetUuidStateNotificationEvent".to_string(),
            abstract_event,
            trashed,
        }
    }
}
