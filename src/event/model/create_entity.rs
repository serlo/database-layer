use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntity {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub entity_id: i32,
}

use crate::event::model::CommonEventData;

impl CreateEntity {
    pub fn build(data: CommonEventData) -> CreateEntity {
        CreateEntity {
            __typename: "CreateEntityNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            entity_id: data.uuid_id,
        }
    }
}
