use crate::event::model::CommonEventData;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaxonomyParent {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub actor_id: i32,
    pub child_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
}

impl SetTaxonomyParent {
    pub fn build(data: CommonEventData, from: Option<i32>, to: Option<i32>) -> SetTaxonomyParent {
        SetTaxonomyParent {
            __typename: "SetTaxonomyParentNotificationEvent".to_string(),
            id: data.id,
            instance: data.instance,
            date: data.date,
            object_id: data.uuid_id,
            actor_id: data.actor_id,
            child_id: data.uuid_id,
            parent_id: to,
            previous_parent_id: from,
        }
    }
}
