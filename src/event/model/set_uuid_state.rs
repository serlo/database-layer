use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    trashed: bool,
}

impl From<AbstractEvent> for SetUuidState {
    fn from(abstract_event: AbstractEvent) -> Self {
        let trashed = abstract_event.raw_typename == "uuid/trash";

        SetUuidState {
            abstract_event,

            trashed,
        }
    }
}
