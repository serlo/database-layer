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
    pub fn new(abstract_event: AbstractEvent) -> Self {
        SetLicense { abstract_event }
    }
}
