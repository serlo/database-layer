use serde::Serialize;

use super::abstract_event::AbstractEvent;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyTerm {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    taxonomy_term_id: i32,
}

impl From<AbstractEvent> for TaxonomyTerm {
    fn from(abstract_event: AbstractEvent) -> Self {
        let taxonomy_term_id = abstract_event.object_id;

        TaxonomyTerm {
            abstract_event,

            taxonomy_term_id,
        }
    }
}
