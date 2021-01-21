use anyhow::Result;
use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unsupported {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub r#type: String,
    pub error: String,
}

impl Unsupported {
    pub async fn fetch(abstract_event: AbstractEvent) -> Result<Self> {
        let r#type = abstract_event.name.to_string();

        Ok(Unsupported {
            abstract_event,

            r#type,
            error: "unsupported".to_string(),
        })
    }
}
