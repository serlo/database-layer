use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::event::Event;
use super::event_type::RawEventType;
use super::EventError;
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::notification::{Notifications, NotificationsError};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentEvent {
    thread_id: i32,
    comment_id: i32,
}

impl TryFrom<&AbstractEvent> for CreateCommentEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let thread_id = abstract_event.uuid_parameters.try_get("discussion")?;
        let comment_id = abstract_event.object_id;

        Ok(CreateCommentEvent {
            thread_id,
            comment_id,
        })
    }
}

pub struct CreateCommentEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    object_id: i32,
    instance_id: i32,
    thread_id: i32,
}

impl CreateCommentEventPayload {
    pub fn new(thread_id: i32, object_id: i32, actor_id: i32, instance_id: i32) -> Self {
        let raw_typename = RawEventType::CreateComment;

        CreateCommentEventPayload {
            raw_typename,
            actor_id,
            object_id,
            instance_id,
            thread_id,
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
            self.object_id,
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

        // insert event_parameter
        sqlx::query!(
            r#"
                INSERT INTO event_parameter (log_id, name_id)
                    SELECT ?, id
                    FROM event_parameter_name
                    WHERE name = ?
            "#,
            event_log_id,
            "discussion"
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
                INSERT INTO event_parameter_uuid (uuid_id, event_parameter_id)
                    VALUES (? , ?)
            "#,
            self.thread_id,
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

    use super::{CreateCommentEvent, CreateCommentEventPayload};
    use crate::create_database_pool;
    use crate::datetime::DateTime;
    use crate::event::{AbstractEvent, ConcreteEvent, Event};

    #[actix_rt::test]
    async fn create_comment() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_thread_state_event = CreateCommentEventPayload::new(16740, 18932, 10, 1);

        let event = set_thread_state_event.save(&mut transaction).await.unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut transaction)
                .await
                .unwrap();

        assert_eq!(event, persisted_event);

        if let Event {
            abstract_event:
                AbstractEvent {
                    actor_id: 10,
                    object_id: 18932,
                    date,
                    uuid_parameters,
                    ..
                },
            concrete_event:
                ConcreteEvent::CreateComment(CreateCommentEvent {
                    thread_id: 16740,
                    comment_id: 18932,
                }),
        } = event
        {
            assert_eq!(uuid_parameters.get("discussion").unwrap(), 16740);
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }
}
