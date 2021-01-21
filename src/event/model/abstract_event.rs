use async_trait::async_trait;
use serde::Serialize;
use sqlx::MySqlPool;

use super::event_type::EventType;
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEvent {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EventType,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub actor_id: i32,
    pub object_id: i32,

    #[serde(skip)]
    pub parameter_uuid_id: i32,
    #[serde(skip)]
    pub name: String,
}

#[async_trait]
pub trait FromAbstractEvent {
    async fn fetch(abstract_event: AbstractEvent, pool: &MySqlPool) -> Result<Self, EventError>
    where
        Self: Sized;
}
