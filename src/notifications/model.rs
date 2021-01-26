use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::Serialize;
use thiserror::Error;

use crate::database::Executor;
use crate::event::{AbstractEvent, Event};
use crate::subscriptions::{Subscriptions, SubscriptionsError};
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifications {
    pub user_id: i32,
    pub notifications: Vec<Notification>,
}

#[derive(Error, Debug)]
pub enum NotificationsError {
    #[error("Navigation cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

#[derive(Serialize)]
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
                    WHERE n.user_id = ?
                    ORDER BY n.date DESC, n.id DESC
            "#,
            user_id
        )
        .fetch_all(executor)
        .await
        .map_err(|inner| NotificationsError::DatabaseError { inner })?;

        let notifications = notifications
            .iter()
            .map(|child| Notification {
                id: child.id as i32,
                unread: child.seen == 0,
                event_id: child.event_log_id as i32,
            })
            .collect();

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
        let mut transaction = executor
            .begin()
            .await
            .map_err(|inner| NotificationsError::DatabaseError { inner })?;

        let AbstractEvent {
            actor_id,
            object_id,
            ..
        } = event.abstract_event;

        let mut subscribers: HashSet<Subscriber> = HashSet::new();

        let mut object_ids = vec![object_id];
        object_ids.extend(event.abstract_event.uuid_parameters.values());

        for object_id in object_ids {
            let subscriptions =
                Subscriptions::fetch_by_object_via_transaction(object_id, &mut transaction)
                    .await
                    .map_err(|error| match error {
                        SubscriptionsError::DatabaseError { inner } => {
                            NotificationsError::DatabaseError { inner }
                        }
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

        transaction
            .commit()
            .await
            .map_err(|inner| NotificationsError::DatabaseError { inner })?;

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
        let mut transaction = executor
            .begin()
            .await
            .map_err(|inner| NotificationsError::DatabaseError { inner })?;

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
        .await
        .map_err(|inner| NotificationsError::DatabaseError { inner })?;
        sqlx::query!(
            r#"
                INSERT INTO notification_event (notification_id, event_log_id)
                    SELECT LAST_INSERT_ID(), ?
            "#,
            event.abstract_event.id,
        )
        .execute(&mut transaction)
        .await
        .map_err(|inner| NotificationsError::DatabaseError { inner })?;

        transaction
            .commit()
            .await
            .map_err(|inner| NotificationsError::DatabaseError { inner })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Notifications;
    use crate::create_database_pool;
    use crate::event::Event;
    use crate::subscriptions::Subscriptions;

    #[actix_rt::test]
    async fn create_notifications_for_event_without_subscribers() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let event = Event::fetch_via_transaction(38513, &mut transaction)
            .await
            .unwrap();

        // Make sure that the event really has no subscribers.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(
            event.abstract_event.object_id,
            &mut transaction,
        )
        .await
        .unwrap();
        assert!(subscriptions.0.is_empty());

        Notifications::create_notifications(&event, &mut transaction)
            .await
            .unwrap();

        // Make sure that no notifications where created.
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

        let event = Event::fetch_via_transaction(41704, &mut transaction)
            .await
            .unwrap();

        // Make sure that the event has a subscriber.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(
            event.abstract_event.object_id,
            &mut transaction,
        )
        .await
        .unwrap();
        assert_eq!(subscriptions.0.len(), 1);
        let subscriber = subscriptions.0[0].user_id;

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

        // Make sure that the notification was created.
        let notifications = Notifications::fetch_via_transaction(subscriber, &mut transaction)
            .await
            .unwrap();
        let notifications: Vec<_> = notifications
            .notifications
            .iter()
            .filter(|notification| notification.event_id == event.abstract_event.id)
            .collect();
        assert_eq!(notifications.len(), 1);
    }

    #[actix_rt::test]
    async fn create_notifications_for_event_with_uuid_parameters() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let event = Event::fetch_via_transaction(37373, &mut transaction)
            .await
            .unwrap();

        // uuids: 15466, 15468

        // Make sure that the event has no direct subscriber.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(
            event.abstract_event.object_id,
            &mut transaction,
        )
        .await
        .unwrap();
        assert!(subscriptions.0.is_empty());

        // Make sure that the event has indirect subscribers.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(
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

        // Make sure that the notifications were created.
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
}
