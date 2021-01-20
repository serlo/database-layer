use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyParent {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,

    pub child_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_parent_id: Option<i32>,
    pub parent_id: Option<i32>,
}

impl SetTaxonomyParent {
    pub fn new(
        abstract_event: AbstractEvent,
        from: Option<i32>,
        to: Option<i32>,
    ) -> SetTaxonomyParent {
        SetTaxonomyParent {
            __typename: "SetTaxonomyParentNotificationEvent".to_string(),
            child_id: abstract_event.object_id,
            parent_id: to,
            previous_parent_id: from,
            abstract_event,
        }
    }
}
