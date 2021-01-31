use serde::Serialize;
use std::convert::TryFrom;

use super::abstract_event::AbstractEvent;
use super::event_type::EventType;
use super::EventError;
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::event::Event;
use crate::notifications::{Notifications, NotificationsError};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadEvent {
    object_id: i32,
    thread_id: i32,
}

impl TryFrom<&AbstractEvent> for CreateThreadEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let object_id = abstract_event.uuid_parameters.try_get("on")?;
        let thread_id = abstract_event.object_id;

        Ok(CreateThreadEvent {
            object_id,
            thread_id,
        })
    }
}

pub struct CreateThreadEventPayload {
    __typename: EventType,
    raw_typename: String,
    actor_id: i32,
    thread_id: i32,
    instance_id: i32,
    uuid_parameter: i32,
}

impl CreateThreadEventPayload {
    pub fn new(thread_id: i32, object_id: i32, actor_id: i32, instance_id: i32) -> Self {
        let raw_typename = "discussion/create".to_string();

        CreateThreadEventPayload {
            __typename: EventType::CreateThread,
            raw_typename,
            actor_id,
            thread_id,
            instance_id,
            uuid_parameter: object_id,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        // insert event_log
        sqlx::query!(
            r#"
                INSERT INTO event_log (actor_id, event_id, uuid_id, instance_id, date)
                    SELECT ?, id, ?, ?, ?
                    FROM event
                    WHERE name = ?
            "#,
            self.actor_id,
            self.thread_id,
            self.instance_id,
            DateTime::now(),
            self.raw_typename,
        )
        .execute(&mut transaction)
        .await?;
        let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?;
        let event_log_id = value.id as i32;

        // TODO: maybe move parameter setting to own function
        // insert event_parameter
        sqlx::query!(
            r#"
                INSERT INTO event_parameter ( log_id , name_id )
                    SELECT ?, id
                    FROM event_parameter_name
                    WHERE name = ?
            "#,
            event_log_id,
            "on"
        )
        .execute(&mut transaction)
        .await?;

        let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?;
        let parameter_id = value.id;

        // insert event_parameter_uuid
        sqlx::query!(
            r#"
                INSERT INTO event_parameter_uuid ( uuid_id, event_parameter_id )
                    VALUES ( ? , ? )
            "#,
            self.uuid_parameter,
            parameter_id
        )
        .execute(&mut transaction)
        .await?;

        let event = Event::fetch_via_transaction(event_log_id, &mut transaction).await?;

        Notifications::create_notifications(&event, &mut transaction)
            .await
            .map_err(|error| match error {
                NotificationsError::DatabaseError { inner } => EventError::from(inner),
            })?;

        transaction.commit().await?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::CreateThreadEventPayload;
    use crate::create_database_pool;
    use crate::datetime::DateTime;
    use crate::event::{AbstractEvent, ConcreteEvent, CreateThreadEvent, Event};

    #[actix_rt::test]
    async fn create_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let create_thread_event = CreateThreadEventPayload::new(16740, 1292, 10, 1);

        let event = create_thread_event.save(&mut transaction).await.unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut transaction)
                .await
                .unwrap();

        assert_eq!(event, persisted_event);

        if let Event {
            abstract_event:
                AbstractEvent {
                    actor_id: 10,
                    object_id: 16740,
                    date,
                    uuid_parameters,
                    ..
                },
            concrete_event:
                ConcreteEvent::CreateThread(CreateThreadEvent {
                    thread_id: 16740,
                    object_id: 1292,
                }),
        } = event
        {
            assert_eq!(uuid_parameters.get(&"on").unwrap(), 1292);
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }
}
