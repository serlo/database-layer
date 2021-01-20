use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyTerm {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub taxonomy_term_id: i32,
}

impl CreateTaxonomyTerm {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateTaxonomyTerm {
            taxonomy_term_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
