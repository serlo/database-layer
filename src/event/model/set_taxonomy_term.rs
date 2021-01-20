use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyTerm {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub taxonomy_term_id: i32,
}

impl SetTaxonomyTerm {
    pub fn new(abstract_event: AbstractEvent) -> SetTaxonomyTerm {
        SetTaxonomyTerm {
            __typename: "SetTaxonomyTermNotificationEvent".to_string(),
            taxonomy_term_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
