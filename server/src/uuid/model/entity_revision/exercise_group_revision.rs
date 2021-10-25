use serde::Serialize;

use super::abstract_entity_revision::AbstractEntityRevision;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseGroupRevision {
    cohesive: bool,
    content: String,
}

impl From<&AbstractEntityRevision> for ExerciseGroupRevision {
    fn from(abstract_entity_revision: &AbstractEntityRevision) -> Self {
        let content = abstract_entity_revision.fields.get_or("description", "");
        let cohesive = abstract_entity_revision.fields.get_or("cohesive", "false") == "true";

        ExerciseGroupRevision {
            cohesive,
            content: if content.is_empty() {
                String::from(r#"{"plugin":"rows","state":[{"plugin":"text"}]}"#)
            } else {
                content
            },
        }
    }
}
