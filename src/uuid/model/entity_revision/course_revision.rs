use serde::Serialize;

use super::abstract_entity_revision::AbstractEntityRevision;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseRevision {
    title: String,
    content: String,
    meta_description: String,
}

impl From<&AbstractEntityRevision> for CourseRevision {
    fn from(abstract_entity_revision: &AbstractEntityRevision) -> Self {
        let title = abstract_entity_revision.fields.get_or("title", "");
        let content = abstract_entity_revision.fields.get_or("description", "");
        let meta_description = abstract_entity_revision
            .fields
            .get_or("meta_description", "");

        CourseRevision {
            title,
            content,
            meta_description,
        }
    }
}
