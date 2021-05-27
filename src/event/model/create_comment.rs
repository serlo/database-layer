use std::collections::HashMap;
use std::convert::TryFrom;

use serde::Serialize;

use crate::database::Executor;

use super::{AbstractEvent, Event, EventError, EventPayload, RawEventType};

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

        Ok(Self {
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

        Self {
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

        let event = EventPayload::new(
            self.raw_typename.clone(),
            self.actor_id,
            self.object_id,
            self.instance_id,
            HashMap::new(),
            [("discussion".to_string(), self.thread_id)]
                .iter()
                .cloned()
                .collect::<HashMap<String, i32>>(),
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

    use crate::create_database_pool;
    use crate::datetime::DateTime;
    use crate::event::{AbstractEvent, ConcreteEvent, Event};

    use super::{CreateCommentEvent, CreateCommentEventPayload};

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
