use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::event_type::EventType;
use super::EventError;
use crate::database::{Acquire, Connection, Executor, Transaction};
use crate::event::model::Event;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUuidState {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

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
    trashed: bool,
    instance: String,
}

impl SetUuidStateEventPayload {
    pub fn new(trashed: bool, actor_id: i32, object_id: i32, instance: &str) -> Self {
        // TODO: handle datetime yourself

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
            trashed,
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
        sqlx::query!(
            r#"
                INSERT INTO event_log (actor_id, event_id, uuid_id, instance_id)
                    SELECT ?, e.id, ?, i.id
                    FROM event e
                    JOIN instance i
                    WHERE e.name = ? AND i.subdomain = ?
            "#,
            self.actor_id,
            self.object_id,
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
    use crate::create_database_pool;
    use crate::event::model::set_uuid_state::SetUuidStateEventPayload;
    use crate::event::model::Event;

    #[actix_rt::test]
    async fn trigger_trash_event() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let set_uuid_state_event = SetUuidStateEventPayload::new(true, 1, 1855, "de");

        let event = set_uuid_state_event.save(&mut transaction).await.unwrap();
        if let Event::SetUuidState(set_uuid_state_event) = event {
            let persisted_event =
                Event::fetch(set_uuid_state_event.abstract_event.id, &mut transaction)
                    .await
                    .unwrap();
        // assert_eq!(persisted_event, event);
        } else {
            panic!("Wrong event type");
        }
    }
}
