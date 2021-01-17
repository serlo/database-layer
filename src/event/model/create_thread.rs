use crate::event::model::CommonEventData;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThread {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub thread_id: i32,
}

impl CreateThread {
    pub fn build(data: CommonEventData) -> CreateThread {
        CreateThread {
            __typename: "CreateThreadNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            // uses "on" parameter
            object_id: data.parameter_uuid_id.unwrap_or(data.uuid_id),
            actor_id: data.actor_id,
            thread_id: data.uuid_id,
        }
    }
}
