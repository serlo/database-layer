use std::collections::HashMap;

use serde::Serialize;

use super::event_type::EventType;
use super::EventError;
use crate::datetime::DateTime;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEvent {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EventType,
    pub id: i32,
    pub instance: String,
    pub date: DateTime,
    pub actor_id: i32,
    pub object_id: i32,

    #[serde(skip)]
    pub raw_typename: String,
    #[serde(skip)]
    pub string_parameters: EventStringParameters,
    #[serde(skip)]
    pub uuid_parameters: EventUuidParameters,
}

impl AbstractEvent {
    // pub async fn save(&self, transaction: &Transaction<'_, MySql>) -> Result<(), EventError> {
    //     // TODO
    //
    //     // TODO: add create_event method (possibly rather for the concrete Event Types) -> see EventManager::logEvent
    //     // TODO: this triggers a 'log' event -> see EventManagerListener::onLog
    //     // TODO: this will create some notifications -> see NotificationManager::createNotification
    //     // TODO: this will trigger a 'create' event which sets the corresponding cache in legacy
    //
    //
    // }
}

#[derive(Debug, PartialEq)]
pub struct EventStringParameters(pub HashMap<String, String>);

impl EventStringParameters {
    pub fn get_or(&self, name: &str, default: &str) -> String {
        self.0
            .get(name)
            .map(|value| value.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct EventUuidParameters(pub HashMap<String, i32>);

impl EventUuidParameters {
    pub fn get(&self, name: &str) -> Option<i32> {
        self.0.get(name).copied()
    }

    pub fn try_get(&self, name: &str) -> Result<i32, EventError> {
        self.get(name).ok_or(EventError::MissingRequiredField)
    }
}
