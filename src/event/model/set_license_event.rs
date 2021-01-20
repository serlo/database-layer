use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLicense {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub repository_id: i32,
}

impl SetLicense {
    pub fn new(abstract_event: AbstractEvent) -> SetLicense {
        SetLicense {
            __typename: "SetLicenseNotificationEvent".to_string(),
            abstract_event,
        }
    }
}
