use std::collections::HashMap;

use serde::Serialize;

use super::event_type::{EventType, RawEventType};
use super::EventError;
use crate::datetime::DateTime;
use crate::instance::Instance;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEvent {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EventType,
    pub id: i32,
    pub instance: Instance,
    pub date: DateTime,
    pub actor_id: i32,
    pub object_id: i32,

    #[serde(skip)]
    pub raw_typename: RawEventType,
    #[serde(skip)]
    pub string_parameters: EventStringParameters,
    #[serde(skip)]
    pub uuid_parameters: EventUuidParameters,
}

#[derive(Debug, Eq, PartialEq)]
pub struct EventStringParameters(pub HashMap<String, String>);

impl EventStringParameters {
    pub fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }

    pub fn get_or(&self, name: &str, default: &str) -> String {
        self.0
            .get(name)
            .map(|value| value.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct EventUuidParameters(pub HashMap<String, i32>);

impl EventUuidParameters {
    pub fn get(&self, name: &str) -> Option<i32> {
        self.0.get(name).copied()
    }

    pub fn try_get(&self, name: &str) -> Result<i32, EventError> {
        self.get(name).ok_or(EventError::MissingRequiredField)
    }

    pub fn values(&self) -> Vec<i32> {
        self.0.values().copied().collect()
    }
}

impl AbstractEvent {
    pub async fn fetch<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        id: i32,
        acquire_from: A,
    ) -> Result<Self, EventError> {
        let mut transaction = acquire_from.begin().await?;

        let event = sqlx::query!(
            r#"
                SELECT l.id, l.actor_id, l.uuid_id, l.date, i.subdomain, e.name
                    FROM event_log l
                    LEFT JOIN event_parameter p ON l.id = p.log_id
                    JOIN instance i ON l.instance_id = i.id
                    JOIN event e ON l.event_id = e.id
                    WHERE l.id = ?
            "#,
            id
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => EventError::NotFound,
            error => error.into(),
        })?;
        let string_parameters = sqlx::query!(
            r#"
                SELECT n.name, s.value
                    FROM event_parameter p
                    JOIN event_parameter_name n ON n.id = p.name_id
                    JOIN event_parameter_string s ON s.event_parameter_id = p.id
                    WHERE p.name_id = n.id AND p.log_id = ?
            "#,
            id
        )
        .fetch_all(&mut *transaction)
        .await?;
        let uuid_parameters = sqlx::query!(
            r#"
                SELECT n.name, u.uuid_id
                    FROM event_parameter p
                    JOIN event_parameter_name n ON n.id = p.name_id
                    JOIN event_parameter_uuid u ON u.event_parameter_id = p.id
                    WHERE p.name_id = n.id AND p.log_id = ?
            "#,
            id
        )
        .fetch_all(&mut *transaction)
        .await?;

        let raw_typename: RawEventType = event.name.parse().map_err(|_| EventError::InvalidType)?;
        let uuid_id = event.uuid_id as i32;

        let string_parameters = string_parameters
            .into_iter()
            .map(|param| (param.name, param.value))
            .collect();
        let string_parameters = EventStringParameters(string_parameters);

        let uuid_parameters = uuid_parameters
            .into_iter()
            .map(|param| (param.name, param.uuid_id as i32))
            .collect();
        let uuid_parameters = EventUuidParameters(uuid_parameters);

        Ok(AbstractEvent {
            __typename: raw_typename.clone().into(),
            id: event.id as i32,
            instance: event
                .subdomain
                .parse()
                .map_err(|_| EventError::InvalidInstance)?,
            date: event.date.into(),
            actor_id: event.actor_id as i32,
            object_id: uuid_id,
            raw_typename,

            string_parameters,
            uuid_parameters,
        })
    }
}
