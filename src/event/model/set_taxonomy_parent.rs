use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyParent {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    child_id: i32,
    previous_parent_id: Option<i32>,
    parent_id: Option<i32>,
}

impl TryFrom<AbstractEvent> for SetTaxonomyParent {
    type Error = EventError;

    fn try_from(abstract_event: AbstractEvent) -> Result<Self, Self::Error> {
        let child_id = abstract_event.object_id;
        let previous_parent_id = abstract_event.uuid_parameters.get("from");
        let parent_id = abstract_event.uuid_parameters.get("to");

        Ok(SetTaxonomyParent {
            abstract_event,

            child_id,
            previous_parent_id,
            parent_id,
        })
    }
}
