use super::event::AbstractEvent;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectRevision {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub repository_id: i32,
    pub revision_id: i32,
    pub reason: String,
}

impl RejectRevision {
    pub async fn new(abstract_event: AbstractEvent, pool: &MySqlPool) -> Self {
        let repository_id =
            super::event::fetch_parameter_uuid_id(abstract_event.id, "repository", &pool)
                .await
                .unwrap();
        let reason = super::event::fetch_parameter_string(abstract_event.id, "reason", &pool).await;

        RejectRevision {
            __typename: "RejectRevisionNotificationEvent".to_string(),
            repository_id,
            revision_id: abstract_event.object_id,
            reason,
            abstract_event,
        }
    }
}
