use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::event_type::EventType;
use super::{Event, EventError};
use crate::database::Executor;
use crate::datetime::DateTime;
use crate::notifications::{Notifications, NotificationsError};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidStateEvent {
    trashed: bool,
}

impl From<&AbstractEvent> for SetUuidStateEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let trashed = abstract_event.raw_typename == "uuid/trash";

        SetUuidStateEvent { trashed }
    }
}

pub struct SetUuidStateEventPayload {
    __typename: EventType,
    raw_typename: String,
    actor_id: i32,
    object_id: i32,
    instance: String,
}

impl SetUuidStateEventPayload {
    pub fn new(trashed: bool, actor_id: i32, object_id: i32, instance: &str) -> Self {
        let raw_typename = if trashed {
            "uuid/trash".to_string()
        } else {
            "uuid/restore".to_string()
        };

        SetUuidStateEventPayload {
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

    use super::{SetUuidStateEvent, SetUuidStateEventPayload};
    use crate::create_database_pool;
    use crate::datetime::DateTime;
    use crate::event::{AbstractEvent, ConcreteEvent, Event};

    #[actix_rt::test]
    async fn trash_event() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_uuid_state_event = SetUuidStateEventPayload::new(true, 1, 1855, "de");

        let event = set_uuid_state_event.save(&mut transaction).await.unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut transaction)
                .await
                .unwrap();

        assert_eq!(event, persisted_event);

        if let Event {
            abstract_event:
                AbstractEvent {
                    actor_id: 1,
                    object_id: 1855,
                    date,
                    ..
                },
            concrete_event: ConcreteEvent::SetUuidState(SetUuidStateEvent { trashed: true }),
        } = event
        {
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }

    #[actix_rt::test]
    async fn restore_event() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_uuid_state_event = SetUuidStateEventPayload::new(false, 1, 1855, "de");

        let event = set_uuid_state_event.save(&mut transaction).await.unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut transaction)
                .await
                .unwrap();

        assert_eq!(event, persisted_event);

        if let Event {
            abstract_event:
                AbstractEvent {
                    actor_id: 1,
                    object_id: 1855,
                    date,
                    ..
                },
            concrete_event: ConcreteEvent::SetUuidState(SetUuidStateEvent { trashed: false }),
        } = event
        {
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }
}
