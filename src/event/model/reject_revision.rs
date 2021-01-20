use anyhow::Result;
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::event::{fetch_parameter_string, fetch_parameter_uuid_id, AbstractEvent};
use crate::event::model::event::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectRevision {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub repository_id: i32,
    pub revision_id: i32,
    pub reason: String,
}

impl RejectRevision {
    pub async fn new(abstract_event: AbstractEvent, pool: &MySqlPool) -> Result<Self> {
        let repository_id_fut = fetch_parameter_uuid_id(abstract_event.id, "repository", pool);
        let reason_fut = fetch_parameter_string(abstract_event.id, "reason", pool);
        let revision_id = abstract_event.object_id;

        if let (Some(repository_id), reason) = try_join!(repository_id_fut, reason_fut)? {
            Ok(RejectRevision {
                abstract_event,

                repository_id,
                revision_id,
                reason: reason.unwrap_or_else(|| "".to_string()),
            })
        } else {
            Err(anyhow::Error::new(EventError::MissingField {
                id: abstract_event.id,
            }))
        }
    }
}
