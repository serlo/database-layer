use std::convert::TryFrom;

use serde::Serialize;

use super::{AbstractEvent, Event, EventError, EventPayload, RawEventType};
use crate::{database::Executor, instance::Instance};

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionEvent {
    repository_id: i32,
    revision_id: i32,
    reason: String,
}

impl TryFrom<&AbstractEvent> for RevisionEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let repository_id = abstract_event.uuid_parameters.try_get("repository")?;
        let revision_id = abstract_event.object_id;
        let reason = abstract_event.string_parameters.get_or("reason", "");

        Ok(RevisionEvent {
            repository_id,
            revision_id,
            reason,
        })
    }
}

pub struct RevisionEventPayload {
    raw_typename: RawEventType,
    user_id: i32,
    repository_id: i32,
    revision_id: i32,
    reason: String,
    instance: Instance,
}

impl RevisionEventPayload {
    pub fn new(
        rejected: bool,
        user_id: i32,
        repository_id: i32,
        revision_id: i32,
        reason: String,
        instance: Instance,
    ) -> Self {
        let raw_typename = if rejected {
            RawEventType::RejectRevision
        } else {
            RawEventType::CheckoutRevision
        };

        Self {
            raw_typename,
            user_id,
            repository_id,
            revision_id,
            reason,
            instance,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let event = EventPayload::new(
            self.raw_typename.clone(),
            self.user_id,
            self.revision_id,
            self.instance.fetch_id(&mut *transaction).await?,
            [("reason".to_string(), self.reason.to_string())]
                .iter()
                .cloned()
                .collect(),
            [("repository".to_string(), self.repository_id)]
                .iter()
                .cloned()
                .collect(),
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

    use super::RevisionEventPayload;
    use crate::create_database_pool;
    use crate::datetime::DateTime;
    use crate::event::{AbstractEvent, ConcreteEvent, Event, RevisionEvent};
    use crate::instance::Instance;

    #[actix_rt::test]
    async fn checkout_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let event = RevisionEventPayload::new(
            false,
            1,
            1855,
            30672,
            "Improve explanation".to_string(),
            Instance::De,
        )
        .save(&mut transaction)
        .await
        .unwrap();
        let persisted_event =
            Event::fetch_via_transaction(event.abstract_event.id, &mut transaction)
                .await
                .unwrap();

        assert_eq!(event, persisted_event);

        if let Event {
            abstract_event:
                AbstractEvent {
                    actor_id: 1,
                    object_id: 30672,
                    date,
                    string_parameters,
                    uuid_parameters,
                    ..
                },
            concrete_event:
                ConcreteEvent::CheckoutRevision(RevisionEvent {
                    repository_id: 1855,
                    revision_id: 30672,
                    reason,
                }),
        } = event
        {
            assert_eq!(reason, "Improve explanation".to_string());
            assert_eq!(string_parameters.get("reason").unwrap(), reason);
            assert_eq!(uuid_parameters.get("repository").unwrap(), 1855);
            assert!(DateTime::now().signed_duration_since(date) < Duration::minutes(1))
        } else {
            panic!("Event does not fulfill assertions: {:?}", event)
        }
    }
}
