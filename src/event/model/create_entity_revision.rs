use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityRevision {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub entity_id: i32,
    pub entity_revision_id: i32,
}

use crate::event::model::CommonEventData;

impl CreateEntityRevision {
    pub fn build(data: CommonEventData) -> CreateEntityRevision {
        CreateEntityRevision {
            __typename: "CreateEntityRevisionNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            entity_id: data.parameter_uuid_id.unwrap_or(data.uuid_id),
            entity_revision_id: data.uuid_id,
        }
    }
}
