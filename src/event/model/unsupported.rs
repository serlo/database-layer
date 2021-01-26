use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsupportedEvent {
    r#type: String,
    error: String,
}

impl From<&AbstractEvent> for UnsupportedEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let r#type = abstract_event.raw_typename.to_string();

        UnsupportedEvent {
            r#type,
            error: "unsupported".to_string(),
        }
    }
}
