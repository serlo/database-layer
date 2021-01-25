use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::event_type::EventType;
use super::{Event, EventError};
use crate::database::Executor;
use crate::datetime::DateTime;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    trashed: bool,
}

impl From<AbstractEvent> for SetUuidState {
    fn from(abstract_event: AbstractEvent) -> Self {
        let trashed = abstract_event.raw_typename == "uuid/trash";

        SetUuidState {
            abstract_event,

            trashed,
        }
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
        let mut transaction = executor
            .begin()
            .await
            .map_err(|inner| EventError::DatabaseError { inner })?;
        let current_datetime = DateTime::now();
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
            current_datetime,
            self.raw_typename,
            self.instance
        )
        .execute(&mut transaction)
        .await
        .map_err(|inner| EventError::DatabaseError { inner })?;
        let value = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await
            .map_err(|inner| EventError::DatabaseError { inner })?;
        let event = Event::fetch(value.id as i32, &mut transaction).await?;
        transaction
            .commit()
            .await
            .map_err(|inner| EventError::DatabaseError { inner })?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::SetUuidState;
    use crate::create_database_pool;
    use crate::datetime::DateTime;
    use crate::event::model::abstract_event::AbstractEvent;
    use crate::event::{Event, SetUuidStateEventPayload};

    #[actix_rt::test]
    async fn trigger_trash_event() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let actor_id = 1;
        let object_id = 1855;
        let instance = "de".to_string();

        let set_uuid_state_event =
            SetUuidStateEventPayload::new(true, actor_id, object_id, &instance);

        let event = set_uuid_state_event.save(&mut transaction).await.unwrap();
        let persisted_event = Event::fetch(event.get_id(), &mut transaction)
            .await
            .unwrap();
        assert_eq!(persisted_event, event);

        if let Event::SetUuidState(SetUuidState {
            abstract_event:
                AbstractEvent {
                    actor_id: 1,
                    object_id: 1855,
                    date,
                    ..
                },
            trashed: true,
            ..
        }) = persisted_event
        {
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", persisted_event)
        }
    }
}
