use crate::event::model::CommonEventData;
use serde::Serialize;

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
    pub fn build(event_data: CommonEventData) -> Unsupported {
        Unsupported {
            id: event_data.id,
            date: event_data.date,
            instance: event_data.instance,
            object_id: event_data.uuid_id,
            r#type: event_data.name,
            error: String::from("unsupported"),
        }
    }
}
