use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyTerm {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub taxonomy_term_id: i32,
}

impl CreateTaxonomyTerm {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateTaxonomyTerm {
            __typename: "CreateTaxonomyTermNotificationEvent".to_string(),
            taxonomy_term_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
