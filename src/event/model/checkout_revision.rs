use anyhow::Result;
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::event::{fetch_string_parameter, fetch_uuid_parameter, AbstractEvent};
use crate::event::model::event::EventError;

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
    pub async fn fetch(abstract_event: AbstractEvent, pool: &MySqlPool) -> Result<Self> {
        let repository_id_fut = fetch_uuid_parameter(abstract_event.id, "repository", pool);
        let reason_fut = fetch_string_parameter(abstract_event.id, "reason", pool);
        let revision_id = abstract_event.object_id;

        if let (Some(repository_id), reason) = try_join!(repository_id_fut, reason_fut)? {
            Ok(CheckoutRevision {
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
