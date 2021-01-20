use serde::Serialize;
use sqlx::MySqlPool;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyParent {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub child_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_parent_id: Option<i32>,
    pub parent_id: Option<i32>,
}

impl SetTaxonomyParent {
    pub async fn new(abstract_event: AbstractEvent, pool: &MySqlPool) -> Self {
        let from = super::event::fetch_parameter_uuid_id(abstract_event.id, "from", &pool).await;
        let to = super::event::fetch_parameter_uuid_id(abstract_event.id, "to", &pool).await;

        SetTaxonomyParent {
            child_id: abstract_event.object_id,
            parent_id: to,
            previous_parent_id: from,
            abstract_event,
        }
    }
}
