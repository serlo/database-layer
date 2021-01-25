use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unsupported {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    r#type: String,
    error: String,
}

impl From<AbstractEvent> for Unsupported {
    fn from(abstract_event: AbstractEvent) -> Self {
        let r#type = abstract_event.raw_typename.to_string();

        Unsupported {
            abstract_event,

            r#type,
            error: "unsupported".to_string(),
        }
    }
}
