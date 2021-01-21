use anyhow::Result;
use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyTerm {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub taxonomy_term_id: i32,
}

impl SetTaxonomyTerm {
    pub async fn fetch(abstract_event: AbstractEvent) -> Result<Self> {
        let taxonomy_term_id = abstract_event.object_id;

        Ok(SetTaxonomyTerm {
            abstract_event,

            taxonomy_term_id,
        })
    }
}
