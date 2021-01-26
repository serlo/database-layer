use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLicenseEvent {
    repository_id: i32,
}

impl From<&AbstractEvent> for SetLicenseEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let repository_id = abstract_event.object_id;

        SetLicenseEvent { repository_id }
    }
}
