use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyLink {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub parent_id: i32,
    pub child_id: i32,
}

impl CreateTaxonomyLink {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateTaxonomyLink {
            __typename: "CreateTaxonomyLinkNotificationEvent".to_string(),
            parent_id: abstract_event.object_id,
            // uses "object" parameter
            child_id: abstract_event.parameter_uuid_id,
            abstract_event,
        }
    }
}
