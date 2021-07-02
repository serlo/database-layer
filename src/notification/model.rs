use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::database::Executor;
use crate::event::{AbstractEvent, Event};
use crate::subscription::{Subscriptions, SubscriptionsError};
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifications {
    pub user_id: i32,
    pub notifications: Vec<Notification>,
}

#[derive(Error, Debug)]
pub enum NotificationsError {
    #[error("Notifications cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for NotificationsError {
    fn from(inner: sqlx::Error) -> Self {
        NotificationsError::DatabaseError { inner }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub unread: bool,
    pub event_id: i32,
}

struct Subscriber {
    user_id: i32,
    send_email: bool,
}
// We consider two subscribers to be equal when their user_id are equal.
impl PartialEq for Subscriber {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}
impl Eq for Subscriber {}
impl Hash for Subscriber {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
    }
}

impl Notifications {
    pub async fn fetch(
        user_id: i32,
        pool: &MySqlPool,
    ) -> Result<Notifications, NotificationsError> {
        Self::fetch_via_transaction(user_id, pool).await
    }

    pub async fn fetch_via_transaction<'a, E>(
        user_id: i32,
        executor: E,
    ) -> Result<Notifications, NotificationsError>
    where
        E: Executor<'a>,
    {
        let notifications = sqlx::query!(
            r#"
                SELECT n.id, n.seen, e.event_log_id
                    FROM notification n
                    JOIN notification_event e ON n.id = e.notification_id
                    JOIN event_log on event_log.id = e.event_log_id
                    JOIN uuid uuid1 on uuid1.id = event_log.uuid_id
                    LEFT JOIN entity entity1 on entity1.id = event_log.uuid_id
                    LEFT JOIN event_parameter ON event_parameter.log_id = event_log.id
                    LEFT JOIN event_parameter_uuid ON
                      event_parameter_uuid.event_parameter_id = event_parameter.id
                    LEFT JOIN uuid uuid2 on uuid2.id = event_parameter_uuid.uuid_id
                    LEFT JOIN entity entity2 on entity2.id = event_parameter_uuid.uuid_id
                    WHERE n.user_id = ?
                      AND uuid1.discriminator NOT IN ("attachment", "blogPost")
                      AND (uuid2.discriminator IS NULL OR
                        uuid2.discriminator NOT IN ("attachment", "blogPost"))
                      AND (entity1.type_id IS NULL OR entity1.type_id IN (1,2,3,4,5,6,7,8,49,50))
                      AND (entity2.type_id IS NULL OR entity2.type_id IN (1,2,3,4,5,6,7,8,49,50))
                    ORDER BY n.date DESC, n.id DESC
            "#,
            user_id
        )
        .fetch_all(executor)
        .await?;

        let mut notifications: Vec<Notification> = notifications
            .iter()
            .map(|child| Notification {
                id: child.id as i32,
                unread: child.seen == 0,
                event_id: child.event_log_id as i32,
            })
            .collect();
        notifications.dedup_by(|n1, n2| n1.id == n2.id);

        Ok(Notifications {
            user_id,
            notifications,
        })
    }

    pub async fn create_notifications<'a, E>(
        event: &Event,
        executor: E,
    ) -> Result<(), NotificationsError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let AbstractEvent {
            actor_id,
            object_id,
            ..
        } = event.abstract_event;

        let mut subscribers: HashSet<Subscriber> = HashSet::new();

        let mut object_ids = vec![object_id];
        object_ids.extend(event.abstract_event.uuid_parameters.values());

        for object_id in object_ids {
            let subscriptions = Subscriptions::fetch_by_object(object_id, &mut transaction)
                .await
                .map_err(|error| match error {
                    SubscriptionsError::DatabaseError { inner } => NotificationsError::from(inner),
                })?;
            let subscriptions = subscriptions
                .0
                .iter()
                .filter(|subscription| subscription.user_id != actor_id);
            for subscription in subscriptions {
                subscribers.insert(Subscriber {
                    user_id: subscription.user_id,
                    send_email: subscription.send_email,
                });
            }
        }

        for subscriber in subscribers {
            Self::create_notification(event, &subscriber, &mut transaction).await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn create_notification<'a, E>(
        event: &Event,
        subscriber: &Subscriber,
        executor: E,
    ) -> Result<(), NotificationsError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO notification (user_id, date, email)
                    VALUES (?, ?, ?)
            "#,
            subscriber.user_id,
            event.abstract_event.date,
            subscriber.send_email
        )
        .execute(&mut transaction)
        .await?;
        sqlx::query!(
            r#"
                INSERT INTO notification_event (notification_id, event_log_id)
                    SELECT LAST_INSERT_ID(), ?
            "#,
            event.abstract_event.id,
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetNotificationStatePayload {
    pub ids: Vec<i32>,
    pub user_id: i32,
    pub unread: bool,
}

#[derive(Error, Debug)]
pub enum SetNotificationStateError {
    #[error("Notification state cannot be set because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for SetNotificationStateError {
    fn from(inner: sqlx::Error) -> Self {
        SetNotificationStateError::DatabaseError { inner }
    }
}

#[derive(Serialize)]
pub struct SetNotificationStateResponse {
    success: bool,
}

impl Notifications {
    pub async fn set_notification_state<'a, E>(
        payload: SetNotificationStatePayload,
        executor: E,
    ) -> Result<SetNotificationStateResponse, SetNotificationStateError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for id in payload.ids.into_iter() {
            let seen = !payload.unread;
            sqlx::query!(
                r#"
                    UPDATE notification
                        SET seen = ?
                        WHERE seen != ? AND id = ?
                "#,
                seen,
                seen,
                id
            )
            .execute(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(SetNotificationStateResponse { success: true })
    }
}

#[cfg(test)]
mod tests {
    use super::{Notifications, NotificationsError, SetNotificationStatePayload, Subscriber};
    use crate::create_database_pool;
    use crate::database::Executor;
    use crate::event::{EntityLinkEventPayload, Event, SetUuidStateEventPayload};
    use crate::instance::Instance;
    use crate::subscription::Subscriptions;
    use rand::{distributions::Alphanumeric, Rng};

    #[actix_rt::test]
    async fn query_notifications_does_not_return_notifications_with_unsupported_uuid() {
        for uuid_type in ["attachment", "blogPost"].iter() {
            let pool = create_database_pool().await.unwrap();

            let unsupported_uuid =
                sqlx::query!("select id from uuid where discriminator = ?", uuid_type)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
                    .id as i32;

            assert_no_notifications_for(
                |user_id| {
                    EntityLinkEventPayload::new(unsupported_uuid, user_id, user_id, Instance::De)
                },
                format!(
                    "when event_log.uuid_id is unsupported with uuid_type: {}",
                    uuid_type
                ),
                &pool,
            )
            .await
            .unwrap();

            assert_no_notifications_for(
                |user_id| {
                    EntityLinkEventPayload::new(user_id, unsupported_uuid, user_id, Instance::De)
                },
                format!(
                    "when event_parameter_uuid is unsupported with uuid_type: {}",
                    uuid_type
                ),
                &pool,
            )
            .await
            .unwrap();
        }
    }

    #[actix_rt::test]
    async fn query_notifications_does_not_return_notifications_with_unsupported_entity() {
        let pool = create_database_pool().await.unwrap();

        let math_puzzle_id = sqlx::query!("select id from entity where type_id = 39")
            .fetch_one(&pool)
            .await
            .unwrap()
            .id as i32;

        assert_no_notifications_for(
            |user_id| EntityLinkEventPayload::new(math_puzzle_id, user_id, user_id, Instance::De),
            "when event_log.uuid_id is unsupported entity".to_string(),
            &pool,
        )
        .await
        .unwrap();

        assert_no_notifications_for(
            |user_id| EntityLinkEventPayload::new(user_id, math_puzzle_id, user_id, Instance::De),
            "when event_parameter_uuid is unsupported entity".to_string(),
            &pool,
        )
        .await
        .unwrap();
    }

    async fn assert_no_notifications_for<'a, E, F>(
        create_event: F,
        message: String,
        executor: E,
    ) -> Result<(), NotificationsError>
    where
        F: Fn(i32) -> EntityLinkEventPayload,
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let new_user_id = create_new_test_user(&mut transaction).await?;
        let event = create_event(new_user_id)
            .save(&mut transaction)
            .await
            .unwrap();

        Notifications::create_notification(
            &event,
            &Subscriber {
                user_id: new_user_id,
                send_email: false,
            },
            &mut transaction,
        )
        .await?;

        assert_eq!(
            Notifications::fetch_via_transaction(new_user_id, &mut transaction)
                .await?
                .notifications
                .len(),
            0,
            "{}",
            message,
        );

        Ok(())
    }

    #[actix_rt::test]
    async fn set_notification_state_no_id() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Notifications::set_notification_state(
            SetNotificationStatePayload {
                ids: vec![],
                user_id: 1,
                unread: true,
            },
            &mut transaction,
        )
        .await
        .unwrap();
    }

    #[actix_rt::test]
    async fn set_notification_state_single_id_to_read() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Notifications::set_notification_state(
            SetNotificationStatePayload {
                ids: vec![6522],
                user_id: 1,
                unread: false,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that the object was set to read (seen is 1).
        let uuid = sqlx::query!(r#"SELECT seen FROM notification WHERE id = ?"#, 6522)
            .fetch_one(&mut transaction)
            .await
            .unwrap();
        assert!(uuid.seen != 0);
    }

    #[actix_rt::test]
    async fn set_notification_state_single_id_to_unread() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Notifications::set_notification_state(
            SetNotificationStatePayload {
                ids: vec![1293],
                user_id: 1,
                unread: true,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that the object was set to unread (seen is 0).
        let uuid = sqlx::query!(r#"SELECT seen FROM notification WHERE id = ?"#, 1293)
            .fetch_one(&mut transaction)
            .await
            .unwrap();
        assert!(uuid.seen == 0);
    }

    #[actix_rt::test]
    async fn set_notification_state_multiple_ids() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let ids = vec![1293, 1307, 1311];

        Notifications::set_notification_state(
            SetNotificationStatePayload {
                ids: ids.clone(),
                user_id: 1,
                unread: true,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        for id in ids.into_iter() {
            let notification = sqlx::query!(
                r#"
                    SELECT id, seen
                        FROM notification
                        WHERE id = ?
                "#,
                id
            )
            .fetch_one(&mut transaction)
            .await
            .unwrap();

            assert!(notification.seen == 0);
        }
    }

    #[actix_rt::test]
    async fn create_notifications_for_event_without_subscribers() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let event = Event::fetch_via_transaction(38513, &mut transaction)
            .await
            .unwrap();

        // Verify assumption that the event has no subscribers.
        let subscriptions =
            Subscriptions::fetch_by_object(event.abstract_event.object_id, &mut transaction)
                .await
                .unwrap();
        assert!(subscriptions.0.is_empty());

        Notifications::create_notifications(&event, &mut transaction)
            .await
            .unwrap();

        // Verify that no notifications where created.
        let notifications = sqlx::query!(
            r#"SELECT * FROM notification_event WHERE event_log_id = ?"#,
            event.abstract_event.id
        )
        .fetch_all(&mut transaction)
        .await
        .unwrap();
        assert!(notifications.is_empty());
    }

    #[actix_rt::test]
    async fn create_notifications_for_event_without_uuid_parameters() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let other_user = 1;
        let test_user = create_new_test_user(&mut transaction).await.unwrap();

        sqlx::query!(
            r#"
                INSERT INTO subscription (uuid_id, user_id, notify_mailman)
                VALUES (?, ?, 1)
            "#,
            other_user,
            test_user
        )
        .execute(&mut transaction)
        .await
        .unwrap();

        SetUuidStateEventPayload::new(false, other_user, other_user, Instance::De)
            .save(&mut transaction)
            .await
            .unwrap();

        // Verify that the notification was created.
        assert_eq!(
            Notifications::fetch_via_transaction(test_user, &mut transaction)
                .await
                .unwrap()
                .notifications
                .len(),
            1
        );
    }

    #[actix_rt::test]
    async fn create_notifications_for_event_with_uuid_parameters() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let event = Event::fetch_via_transaction(37373, &mut transaction)
            .await
            .unwrap();

        // Verify the assumption that the event has no direct subscriber.
        let subscriptions =
            Subscriptions::fetch_by_object(event.abstract_event.object_id, &mut transaction)
                .await
                .unwrap();
        assert!(subscriptions.0.is_empty());

        // Verify the assumption that the event has indirect subscribers.
        let subscriptions = Subscriptions::fetch_by_object(
            *event
                .abstract_event
                .uuid_parameters
                .values()
                .first()
                .unwrap(),
            &mut transaction,
        )
        .await
        .unwrap();
        assert!(!subscriptions.0.is_empty());

        let subscribers = subscriptions.0.iter().map(|s| s.user_id);

        // Clear notifications for this event.
        sqlx::query!(
            r#"DELETE FROM notification_event WHERE event_log_id = ?"#,
            event.abstract_event.id
        )
        .execute(&mut transaction)
        .await
        .unwrap();

        Notifications::create_notifications(&event, &mut transaction)
            .await
            .unwrap();

        // Verify that the notifications were created.
        for subscriber in subscribers {
            let notifications = Notifications::fetch_via_transaction(subscriber, &mut transaction)
                .await
                .unwrap();
            let notifications: Vec<_> = notifications
                .notifications
                .iter()
                .filter(|notification| notification.event_id == event.abstract_event.id)
                .collect();

            if subscriber == event.abstract_event.actor_id {
                assert!(notifications.is_empty());
            } else {
                assert_eq!(notifications.len(), 1);
            }
        }
    }

    async fn create_new_test_user<'a, E>(executor: E) -> Result<i32, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator) VALUES (0, "user")
            "#
        )
        .execute(&mut transaction)
        .await?;

        let new_user_id = sqlx::query!("SELECT LAST_INSERT_ID() as id FROM uuid")
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        sqlx::query!(
            r#"
                INSERT INTO user (id, username, email, password, token)
                VALUES (?, ?, ?, "", ?)
            "#,
            new_user_id,
            random_string(10),
            random_string(10),
            random_string(10)
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(new_user_id)
    }

    fn random_string(nr: usize) -> String {
        return rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(nr)
            .map(char::from)
            .collect();
    }
}
