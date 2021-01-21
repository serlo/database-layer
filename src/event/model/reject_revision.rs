use async_trait::async_trait;
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::event::Event;
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectRevision {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    repository_id: i32,
    revision_id: i32,
    reason: String,
}

#[async_trait]
impl FromAbstractEvent for RejectRevision {
    async fn fetch(abstract_event: AbstractEvent, pool: &MySqlPool) -> Result<Self, EventError> {
        let repository_id_fut = Event::fetch_uuid_parameter(abstract_event.id, "repository", pool);
        let reason_fut = Event::fetch_string_parameter(abstract_event.id, "reason", pool);
        let revision_id = abstract_event.object_id;

        if let (Some(repository_id), reason) = try_join!(repository_id_fut, reason_fut)? {
            Ok(RejectRevision {
                abstract_event,

                repository_id,
                revision_id,
                reason: reason.unwrap_or_else(|| "".to_string()),
            })
        } else {
            Err(EventError::MissingRequiredField)
        }
    }
}
