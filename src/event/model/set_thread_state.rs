use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::event_type::EventType;
use super::{Event, EventError};
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::notifications::{Notifications, NotificationsError};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadStateEvent {
    thread_id: i32,
    archived: bool,
}

impl From<&AbstractEvent> for SetThreadStateEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let thread_id = abstract_event.object_id;
        let archived = abstract_event.raw_typename == "discussion/comment/archive";

        SetThreadStateEvent {
            thread_id,
            archived,
        }
    }
}

pub struct SetThreadStateEventPayload {
    __typename: EventType,
    raw_typename: String,
    actor_id: i32,
    object_id: i32,
    instance: String,
}

impl SetThreadStateEventPayload {
    pub fn new(archived: bool, actor_id: i32, object_id: i32, instance: &str) -> Self {
        let raw_typename = if archived {
            "discussion/comment/archive".to_string()
        } else {
            "discussion/restore".to_string()
        };

        SetThreadStateEventPayload {
            __typename: EventType::SetThreadState,
            raw_typename,
            actor_id,
            object_id,
            instance: instance.to_string(),
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        sqlx::query!(
            r#"
                INSERT INTO event_log (actor_id, event_id, uuid_id, instance_id, date)
                    SELECT ?, e.id, ?, i.id, ?
                    FROM event e
                    JOIN instance i
                    WHERE e.name = ? AND i.subdomain = ?
            "#,
            self.actor_id,
            self.object_id,
            DateTime::now(),
            self.raw_typename,
            self.instance
        )
        .execute(&mut transaction)
        .await?;
        let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?;
        let event = Event::fetch_via_transaction(value.id as i32, &mut transaction).await?;

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

    use super::{SetThreadStateEvent, SetThreadStateEventPayload};
    use crate::create_database_pool;
    use crate::datetime::DateTime;
    use crate::event::{AbstractEvent, ConcreteEvent, Event};

    #[actix_rt::test]
    async fn archive_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_thread_state_event = SetThreadStateEventPayload::new(true, 16462, 17666, "de");

        let event = set_thread_state_event.save(&mut transaction).await.unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut transaction)
                .await
                .unwrap();

        assert_eq!(event, persisted_event);

        if let Event {
            abstract_event:
                AbstractEvent {
                    actor_id: 16462,
                    object_id: 17666,
                    date,
                    ..
                },
            concrete_event:
                ConcreteEvent::SetThreadState(SetThreadStateEvent {
                    thread_id: 17666,
                    archived: true,
                }),
        } = event
        {
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }

    // unsupported?
    #[actix_rt::test]
    async fn unarchive_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_thread_state_event = SetThreadStateEventPayload::new(false, 15478, 17796, "de");

        let event = set_thread_state_event.save(&mut transaction).await.unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut transaction)
                .await
                .unwrap();

        assert_eq!(event, persisted_event);

        if let Event {
            abstract_event:
                AbstractEvent {
                    actor_id: 15478,
                    object_id: 17796,
                    date,
                    ..
                },
            concrete_event:
                ConcreteEvent::SetThreadState(SetThreadStateEvent {
                    thread_id: 17796,
                    archived: false,
                }),
        } = event
        {
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }
}
