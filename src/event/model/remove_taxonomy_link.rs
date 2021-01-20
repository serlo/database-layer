use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveTaxonomyLink {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub parent_id: i32,
    pub child_id: i32,
}

impl RemoveTaxonomyLink {
    pub fn new(abstract_event: AbstractEvent) -> RemoveTaxonomyLink {
        RemoveTaxonomyLink {
            __typename: "RemoveTaxonomyLinkNotificationEvent".to_string(),
            parent_id: abstract_event.object_id,
            // uses "object" parameter
            child_id: abstract_event.parameter_uuid_id,
            abstract_event,
        }
    }
}
