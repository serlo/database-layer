use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutRevision {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub repository_id: i32,
    pub revision_id: i32,
    pub reason: String,
}

impl CheckoutRevision {
    pub fn new(
        abstract_event: AbstractEvent,
        repository_id: i32,
        reason: String,
    ) -> CheckoutRevision {
        CheckoutRevision {
            __typename: "CheckoutRevisionNotificationEvent".to_string(),
            repository_id,
            reason,
            revision_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
