use std::collections::HashMap;
use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::event::{Event, EventPayload};
use super::event_type::RawEventType;
use super::EventError;
use crate::database::Executor;

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

        Ok(Self {
            object_id,
            thread_id,
        })
    }
}

pub struct CreateThreadEventPayload {
    raw_typename: RawEventType,
    actor_id: i32,
    thread_id: i32,
    instance_id: i32,
    uuid_parameter: i32,
}

impl CreateThreadEventPayload {
    pub fn new(thread_id: i32, object_id: i32, actor_id: i32, instance_id: i32) -> Self {
        let raw_typename = RawEventType::CreateThread;

        Self {
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
        EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.thread_id,
            self.instance_id,
            HashMap::new(),
            [("on".to_string(), self.uuid_parameter)]
                .iter()
                .cloned()
                .collect(),
        )
        .save(executor)
        .await
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

        let event = create_thread_event.save(&mut *transaction).await.unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut *transaction)
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
            assert_eq!(uuid_parameters.get("on").unwrap(), 1292);
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }
}
