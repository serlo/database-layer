use serde::Serialize;

use super::entity_type::EntityType;
use crate::datetime::DateTime;
use crate::instance::Instance;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEntity {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EntityType,
    pub instance: Instance,
    pub date: DateTime,
    pub license_id: i32,
    pub taxonomy_term_ids: Vec<i32>,
    pub canonical_subject_id: Option<i32>,

    pub current_revision_id: Option<i32>,
    pub revision_ids: Vec<i32>,
}
