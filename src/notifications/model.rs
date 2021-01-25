use crate::event::Event;
use crate::subscriptions::{SubscriptionByObject, Subscriptions};
use serde::Serialize;
use sqlx::MySqlPool;
use std::collections::HashSet;
use thiserror::Error;

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

impl Notifications {
    pub async fn fetch(
        user_id: i32,
        pool: &MySqlPool,
    ) -> Result<Notifications, NotificationsError> {
        let notifications_fut = sqlx::query!(
            r#"
                SELECT n.id, n.seen, e.event_log_id
                    FROM notification n
                    JOIN notification_event e ON n.id = e.notification_id
                    WHERE n.user_id = ?
                    ORDER BY n.date DESC, n.id DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|inner| NotificationsError::DatabaseError { inner })?;

        let notifications: Vec<Notification> = notifications_fut
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
    ) -> Result<Vec<Notifications>, NotificationsError> {
        let actor_id = event.get_actor_id();
        let object_id = event.get_object_id();

        let mut subscribers: HashSet<&SubscriptionByObject> = HashSet::new();

        let mut object_ids = vec![object_id];
        object_ids.append(&mut event.get_uuid_parameters().map(|value| *value).collect());

        for object_id in object_ids {
            let subs = Subscriptions::fetch_by_object(object_id, executor)
                .await?
                .subscriptions
                .iter()
                .filter(|subscription| subscription.user_id != actor_id);
            for sub in subs {
                subscribers.insert(sub);
            }
        }

        for sub in subscribers {
            // TODO: notify sub
            let user_id = sub.user_id;
            //         $this->getNotificationManager()->createNotification(
            //         $subscriber,
            //         $eventLog,
            //         $subscription->getNotifyMailman()
            //         );
        }

        // Create Notifications for directly subscribed users

        // /* @var SubscriptionInterface[] $subscriptions */
        // $object = $eventLog->getObject();
        // $subscriptions = $this->getSubscriptionManager()->findSubscriptionsByUuid(
        // $object
        // );
        // $subscribed = [];
        //
        // foreach ($subscriptions as $subscription) {
        //     $subscriber = $subscription->getSubscriber();
        //     // Don't create notifications for myself
        //     if ($subscriber !== $eventLog->getActor()) {
        //         $this->getNotificationManager()->createNotification(
        //         $subscriber,
        //         $eventLog,
        //         $subscription->getNotifyMailman()
        //         );
        //         $subscribed[] = $subscriber;
        //     }
        // }
        //
        // foreach ($eventLog->getParameters() as $parameter) {
        //     if ($parameter->getValue() instanceof UuidInterface) {
        //         /* @var $subscribers UserInterface[] */
        //         $object = $parameter->getValue();
        //         $subscriptions = $this->getSubscriptionManager()->findSubscriptionsByUuid(
        //         $object
        //         );
        //
        //         foreach ($subscriptions as $subscription) {
        //             $subscriber = $subscription->getSubscriber();
        //             if (
        //                 !in_array($subscriber, $subscribed) &&
        //             $subscriber !== $eventLog->getActor()
        //             ) {
        //                 $this->getNotificationManager()->createNotification(
        //                 $subscriber,
        //                 $eventLog,
        //                 $subscription->getNotifyMailman()
        //                 );
        //             }
        //         }
        //     }
        // }
        unimplemented!()
    }
}
