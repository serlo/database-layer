use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveEntityLink {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub child_id: i32,
    pub parent_id: i32,
}

use crate::event::model::CommonEventData;

impl RemoveEntityLink {
    pub fn build(data: CommonEventData) -> RemoveEntityLink {
        RemoveEntityLink {
            __typename: "RemoveEntityLinkNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            child_id: data.uuid_id,
            parent_id: data.parameter_uuid_id.unwrap_or(data.uuid_id),
        }
    }
}
