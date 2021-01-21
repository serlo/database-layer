use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::{AbstractEvent, FromAbstractEvent};
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyTerm {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    taxonomy_term_id: i32,
}

#[async_trait]
impl FromAbstractEvent for SetTaxonomyTerm {
    async fn fetch(abstract_event: AbstractEvent, _pool: &MySqlPool) -> Result<Self, EventError> {
        let taxonomy_term_id = abstract_event.object_id;

        Ok(SetTaxonomyTerm {
            abstract_event,

            taxonomy_term_id,
        })
    }
}
