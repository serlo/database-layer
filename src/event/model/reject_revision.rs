use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectRevision {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub repository_id: i32,
    pub revision_id: i32,
    pub reason: String,
}

use crate::event::model::CommonEventData;
impl RejectRevision {
    pub fn build(data: CommonEventData, repository_id: i32, reason: String) -> RejectRevision {
        RejectRevision {
            __typename: "RejectRevisionNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            repository_id,
            revision_id: data.uuid_id,
            reason,
        }
    }
}
