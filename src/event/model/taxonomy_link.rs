use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyLinkEvent {
    parent_id: i32,
    child_id: i32,
}

impl TryFrom<&AbstractEvent> for TaxonomyLinkEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let parent_id = abstract_event.object_id;
        let child_id = abstract_event.uuid_parameters.try_get("object")?;

        Ok(TaxonomyLinkEvent {
            parent_id,
            child_id,
        })
    }
}
