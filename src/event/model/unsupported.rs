use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unsupported {
    r#type: String,
    error: String,
}

impl From<&AbstractEvent> for Unsupported {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let r#type = abstract_event.raw_typename.to_string();

        Unsupported {
            r#type,
            error: "unsupported".to_string(),
        }
    }
}
