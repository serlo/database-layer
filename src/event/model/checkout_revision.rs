use serde::Serialize;
use sqlx::MySqlPool;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutRevision {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub repository_id: i32,
    pub revision_id: i32,
    pub reason: String,
}

impl CheckoutRevision {
    pub async fn new(abstract_event: AbstractEvent, pool: &MySqlPool) -> Self {
        let repository_id =
            super::event::fetch_parameter_uuid_id(abstract_event.id, "repository", &pool)
                .await
                .unwrap();
        let reason = super::event::fetch_parameter_string(abstract_event.id, "reason", &pool).await;

        CheckoutRevision {
            repository_id,
            reason,
            revision_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
