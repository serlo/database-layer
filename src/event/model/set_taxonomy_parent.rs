use anyhow::Result;
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::event::model::event::{fetch_parameter_uuid_id, AbstractEvent};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyParent {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub child_id: i32,
    pub previous_parent_id: Option<i32>,
    pub parent_id: Option<i32>,
}

impl SetTaxonomyParent {
    pub async fn new(abstract_event: AbstractEvent, pool: &MySqlPool) -> Result<Self> {
        let from_fut = fetch_parameter_uuid_id(abstract_event.id, "from", pool);
        let to_fut = fetch_parameter_uuid_id(abstract_event.id, "to", pool);

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
