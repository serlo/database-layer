use serde::Serialize;

use super::abstract_entity_revision::AbstractEntityRevision;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppletRevision {
    #[serde(flatten)]
    pub abstract_entity_revision: AbstractEntityRevision,

    url: String,
    title: String,
    content: String,
    meta_title: String,
    meta_description: String,
}

impl From<AbstractEntityRevision> for AppletRevision {
    fn from(abstract_entity_revision: AbstractEntityRevision) -> Self {
        let url = abstract_entity_revision.fields.get_or("url", "");
        let title = abstract_entity_revision.fields.get_or("title", "");
        let content = abstract_entity_revision.fields.get_or("content", "");
        let meta_title = abstract_entity_revision.fields.get_or("meta_title", "");
        let meta_description = abstract_entity_revision
            .fields
            .get_or("meta_description", "");

        AppletRevision {
            abstract_entity_revision,

            url,
            title,
            content,
            meta_title,
            meta_description,
        }
    }
}
