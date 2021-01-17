use crate::event::model::CommonEventData;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyTerm {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub taxonomy_term_id: i32,
}

impl CreateTaxonomyTerm {
    pub fn build(data: CommonEventData) -> CreateTaxonomyTerm {
        CreateTaxonomyTerm {
            __typename: "CreateTaxonomyTermNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            taxonomy_term_id: data.uuid_id,
        }
    }
}
