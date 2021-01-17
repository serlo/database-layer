use crate::event::model::CommonEventData;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadState {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub thread_id: i32,
    pub archived: bool,
}

impl SetThreadState {
    pub fn build(data: CommonEventData, archived: bool) -> SetThreadState {
        SetThreadState {
            __typename: "SetThreadStateNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            thread_id: data.parameter_uuid_id.unwrap_or(data.uuid_id),
            archived,
        }
    }
}
