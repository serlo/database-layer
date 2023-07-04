use std::collections::HashMap;

use serde::Serialize;

use super::{AbstractEvent, Event, EventError, EventPayload, RawEventType};
use crate::database::Executor;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetThreadStateEvent {
    thread_id: i32,
    archived: bool,
}

impl From<&AbstractEvent> for SetThreadStateEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let thread_id = abstract_event.object_id;
        let archived = abstract_event.raw_typename == RawEventType::ArchiveThread;

        Self {
            thread_id,
            archived,
        }
    }
}

pub struct SetThreadStateEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    object_id: i32,
}

impl SetThreadStateEventPayload {
    pub fn new(archived: bool, actor_id: i32, object_id: i32) -> Self {
        let raw_typename = if archived {
            RawEventType::ArchiveThread
        } else {
            RawEventType::RestoreThread
        };

        Self {
            raw_typename,
            actor_id,
            object_id,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let result = sqlx::query!(
            r#"SELECT instance_id FROM comment WHERE id = ?"#,
            self.object_id
        )
        .fetch_one(&mut *transaction)
        .await?;

        let event = EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.object_id,
            result.instance_id,
            HashMap::new(),
            HashMap::new(),
        )
        .save(&mut *transaction)
        .await?;

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
    use crate::instance::Instance;

    #[actix_rt::test]
    async fn archive_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_thread_state_event = SetThreadStateEventPayload::new(true, 16462, 17666);

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
                    instance: Instance::De,
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

    #[actix_rt::test]
    async fn restore_thread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_thread_state_event = SetThreadStateEventPayload::new(false, 15478, 17796);

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
                    instance: Instance::De,
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
