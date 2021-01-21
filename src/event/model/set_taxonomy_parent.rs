use async_trait::async_trait;
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::event::Event;
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyParent {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    child_id: i32,
    previous_parent_id: Option<i32>,
    parent_id: Option<i32>,
}

#[async_trait]
impl FromAbstractEvent for SetTaxonomyParent {
    async fn fetch(abstract_event: AbstractEvent, pool: &MySqlPool) -> Result<Self, EventError> {
        let from_fut = Event::fetch_uuid_parameter(abstract_event.id, "from", pool);
        let to_fut = Event::fetch_uuid_parameter(abstract_event.id, "to", pool);

        let (previous_parent_id, parent_id) = try_join!(from_fut, to_fut)?;
        let child_id = abstract_event.object_id;

        Ok(SetTaxonomyParent {
            abstract_event,

            child_id,
            previous_parent_id,
            parent_id,
        })
    }
}
