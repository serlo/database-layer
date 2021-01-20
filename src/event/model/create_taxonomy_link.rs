use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyLink {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub parent_id: i32,
    pub child_id: i32,
}

impl CreateTaxonomyLink {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateTaxonomyLink {
            parent_id: abstract_event.object_id,
            // uses "object" parameter
            child_id: abstract_event.parameter_uuid_id,
            abstract_event,
        }
    }
}
