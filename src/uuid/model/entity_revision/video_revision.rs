use std::convert::From;

use serde::Serialize;

use super::abstract_entity_revision::AbstractEntityRevision;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoRevision {
    #[serde(flatten)]
    pub abstract_entity_revision: AbstractEntityRevision,

    url: String,
    title: String,
    content: String,
}

impl From<AbstractEntityRevision> for VideoRevision {
    fn from(abstract_entity_revision: AbstractEntityRevision) -> Self {
        let url = abstract_entity_revision.fields.get_or("content", "");
        let title = abstract_entity_revision.fields.get_or("title", "");
        let content = abstract_entity_revision.fields.get_or("description", "");

        VideoRevision {
            abstract_entity_revision,

            url,
            title,
            content,
        }
    }
}
