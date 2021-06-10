use std::collections::HashMap;

use serde::Serialize;

use super::{AbstractEvent, Event, EventError, EventPayload, RawEventType};
use crate::{database::Executor, instance::Instance};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidStateEvent {
    trashed: bool,
}

impl From<&AbstractEvent> for SetUuidStateEvent {
    fn from(abstract_event: &AbstractEvent) -> Self {
        let trashed = abstract_event.raw_typename == RawEventType::TrashUuid;

        Self { trashed }
    }
}

pub struct SetUuidStateEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    object_id: i32,
    instance: Instance,
}

impl SetUuidStateEventPayload {
    pub fn new(trashed: bool, actor_id: i32, object_id: i32, instance: Instance) -> Self {
        let raw_typename = if trashed {
            RawEventType::TrashUuid
        } else {
            RawEventType::RestoreUuid
        };

        Self {
            raw_typename,
            actor_id,
            object_id,
            instance,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let instance_id = sqlx::query!(
            r#"SELECT id FROM instance WHERE subdomain = ?"#,
            self.instance
        )
        .fetch_one(&mut transaction)
        .await?;

        let event = EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.object_id,
            instance_id.id,
            HashMap::new(),
            HashMap::new(),
        )
        .save(&mut transaction)
        .await?;

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
    use crate::instance::Instance;

    #[actix_rt::test]
    async fn trash_event() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_uuid_state_event = SetUuidStateEventPayload::new(true, 1, 1855, Instance::De);

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

        let set_uuid_state_event = SetUuidStateEventPayload::new(false, 1, 1855, Instance::De);

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
