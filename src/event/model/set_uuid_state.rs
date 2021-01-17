use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub trashed: bool,
}

use crate::event::model::CommonEventData;

impl SetUuidState {
    pub fn build(data: CommonEventData, trashed: bool) -> SetUuidState {
        SetUuidState {
            __typename: "SetUuidStateNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            trashed,
        }
    }
}
