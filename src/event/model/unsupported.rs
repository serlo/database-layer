use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unsupported {
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub object_id: i32,
    pub r#type: String,
    pub error: String,
}

impl Unsupported {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        Unsupported {
            id: abstract_event.id,
            date: abstract_event.date,
            instance: abstract_event.instance,
            object_id: abstract_event.object_id,
            r#type: abstract_event.name,
            error: "unsupported".to_string(),
        }
    }
}
