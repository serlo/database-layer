use anyhow::Result;
use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLicense {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub repository_id: i32,
}

impl SetLicense {
    pub async fn new(abstract_event: AbstractEvent) -> Result<Self> {
        let repository_id = abstract_event.object_id;

        Ok(SetLicense {
            abstract_event,

            repository_id,
        })
    }
}
