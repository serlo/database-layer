use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidStateEvent {
    trashed: bool,
}

impl From<&AbstractEvent> for SetUuidStateEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let trashed = abstract_event.raw_typename == "uuid/trash";

        SetUuidStateEvent { trashed }
    }
}
