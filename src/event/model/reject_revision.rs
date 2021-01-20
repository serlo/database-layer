use super::event::AbstractEvent;
use serde::Serialize;

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
    pub fn new(
        abstract_event: AbstractEvent,
        repository_id: i32,
        reason: String,
    ) -> RejectRevision {
        RejectRevision {
            __typename: "RejectRevisionNotificationEvent".to_string(),
            repository_id,
            revision_id: abstract_event.object_id,
            reason,
            abstract_event,
        }
    }
}
