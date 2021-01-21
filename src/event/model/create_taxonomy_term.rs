use anyhow::Result;
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
    pub async fn fetch(abstract_event: AbstractEvent) -> Result<Self> {
        let taxonomy_term_id = abstract_event.object_id;

        Ok(CreateTaxonomyTerm {
            abstract_event,

            taxonomy_term_id,
        })
    }
}
