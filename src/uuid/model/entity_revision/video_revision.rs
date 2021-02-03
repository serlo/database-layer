use serde::Serialize;

use super::abstract_entity_revision::AbstractEntityRevision;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoRevision {
    url: String,
    title: String,
    content: String,
}

impl From<&AbstractEntityRevision> for VideoRevision {
    fn from(abstract_entity_revision: &AbstractEntityRevision) -> Self {
        let url = abstract_entity_revision.fields.get_or("content", "");
        let title = abstract_entity_revision.fields.get_or("title", "");
        let content = abstract_entity_revision.fields.get_or("description", "");

        VideoRevision {
            url,
            title,
            content,
        }
    }
}
