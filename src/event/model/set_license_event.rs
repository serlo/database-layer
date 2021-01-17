use crate::event::model::CommonEventData;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLicense {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub repository_id: i32,
}

impl SetLicense {
    pub fn build(data: CommonEventData) -> SetLicense {
        SetLicense {
            __typename: "SetLicenseNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
        }
    }
}
