use crate::subscription::messages::{subscription_set_mutation, subscriptions_query};
use sqlx::MySqlPool;

use crate::datetime::DateTime;

#[derive(Debug, Eq, PartialEq)]
pub struct Subscriptions(pub Vec<Subscription>);

#[derive(Debug, Eq, PartialEq)]
pub struct Subscription {
    pub object_id: i32,
    pub user_id: i32,
    pub send_email: bool,
}

impl Subscriptions {
    pub async fn fetch_by_user<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        user_id: i32,
        acquire_from: A,
    ) -> Result<Self, sqlx::Error> {
        let mut connection = acquire_from.acquire().await?;
        let subscriptions = sqlx::query!(
            r#"
                SELECT s.uuid_id, s.user_id, s.notify_mailman FROM subscription s
                JOIN uuid on uuid.id = s.uuid_id
                LEFT JOIN entity on entity.id = s.uuid_id
                WHERE s.user_id = ?
                    AND uuid.discriminator NOT IN ("attachment", "blogPost")
                    AND (entity.type_id IS NULL OR entity.type_id IN (1,2,3,4,5,6,7,8,49,50))
            "#,
            user_id
        )
        .fetch_all(&mut *connection)
        .await?;

        let subscriptions = subscriptions
            .iter()
            .map(|child| Subscription {
                object_id: child.uuid_id as i32,
                user_id: child.user_id as i32,
                send_email: child.notify_mailman != 0,
            })
            .collect();

        Ok(Subscriptions(subscriptions))
    }

    pub async fn fetch_by_object<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        object_id: i32,
        acquire_from: A,
    ) -> Result<Self, sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;
        let subscriptions = sqlx::query!(
            r#"SELECT uuid_id, user_id, notify_mailman FROM subscription WHERE uuid_id = ?"#,
            object_id
        )
        .fetch_all(&mut *transaction)
        .await?;

        let subscriptions = subscriptions
            .iter()
            .map(|child| Subscription {
                object_id: child.uuid_id as i32,
                user_id: child.user_id as i32,
                send_email: child.notify_mailman != 0,
            })
            .collect();

        Ok(Subscriptions(subscriptions))
    }
}

impl Subscription {
    pub async fn save<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        &self,
        acquire_from: A,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;
        sqlx::query!(
            r#"
                INSERT INTO subscription (uuid_id, user_id, notify_mailman, date)
                    VALUES (?, ?, ?, ?)
                    ON DUPLICATE KEY UPDATE notify_mailman = ?
            "#,
            self.object_id,
            self.user_id,
            self.send_email,
            DateTime::now(),
            self.send_email,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn remove<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        &self,
        acquire_from: A,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;
        sqlx::query!(
            r#"DELETE FROM subscription WHERE uuid_id = ? AND user_id = ?"#,
            self.object_id,
            self.user_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
}

pub async fn fetch_subscriptions_by_user<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
    user_id: i32,
    acquire_from: A,
) -> Result<subscriptions_query::Output, sqlx::Error> {
    let subscriptions = Subscriptions::fetch_by_user(user_id, acquire_from).await?;
    let subscriptions = subscriptions
        .0
        .iter()
        .map(|child| subscriptions_query::SubscriptionByUser {
            object_id: child.object_id,
            send_email: child.send_email,
        })
        .collect();

    Ok(subscriptions_query::Output { subscriptions })
}

impl Subscription {
    pub async fn change_subscription<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &subscription_set_mutation::Payload,
        acquire_from: A,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;

        for id in &payload.ids {
            let subscription = Subscription {
                object_id: *id,
                user_id: payload.user_id,
                send_email: payload.send_email,
            };

            if payload.subscribe {
                subscription.save(&mut *transaction).await?;
            } else {
                subscription.remove(&mut *transaction).await?;
            }
        }
        transaction.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Subscription, Subscriptions};
    use crate::create_database_pool;

    #[actix_rt::test]
    async fn get_subscriptions_does_not_return_unsupported_uuids() {
        for unsupported_uuid in ["blogPost", "attachment"].iter() {
            let pool = create_database_pool().await.unwrap();
            let mut transaction = pool.begin().await.unwrap();

            let uuid_id = sqlx::query!(
                "SELECT id FROM uuid WHERE discriminator = ?",
                unsupported_uuid
            )
            .fetch_one(&mut *transaction)
            .await
            .unwrap()
            .id as i32;

            assert_get_subscriptions_does_not_return(uuid_id, &mut *transaction)
                .await
                .unwrap();
        }
    }

    #[actix_rt::test]
    async fn get_subscriptions_does_not_return_unsupported_entity() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let entity_id = sqlx::query!("SELECT id FROM entity WHERE type_id = 43",)
            .fetch_one(&mut *transaction)
            .await
            .unwrap()
            .id as i32;

        assert_get_subscriptions_does_not_return(entity_id, &mut *transaction)
            .await
            .unwrap();
    }

    async fn assert_get_subscriptions_does_not_return<
        'a,
        A: sqlx::Acquire<'a, Database = sqlx::MySql>,
    >(
        uuid_id: i32,
        acquire_from: A,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;
        let user_id: i32 = 35408;

        sqlx::query!(
            r#"
                INSERT INTO subscription (uuid_id, user_id, notify_mailman)
                VALUES (?, ?, 1)
            "#,
            uuid_id,
            user_id
        )
        .execute(&mut *transaction)
        .await?;

        let subscriptions = Subscriptions::fetch_by_user(user_id, &mut *transaction).await?;

        assert!(!subscriptions.0.iter().any(|sub| sub.object_id == uuid_id));

        Ok(())
    }

    #[actix_rt::test]
    async fn create_subscription_new() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Verify assumption that the user is not subscribed to object
        let subscription = fetch_subscription_by_user_and_object(1, 1555, &mut *transaction)
            .await
            .unwrap();
        assert!(subscription.is_none());

        let subscription = Subscription {
            object_id: 1555,
            user_id: 1,
            send_email: true,
        };
        subscription.save(&mut *transaction).await.unwrap();

        // Verify that user is now subscribed to object
        let new_subscription = fetch_subscription_by_user_and_object(1, 1555, &mut *transaction)
            .await
            .unwrap();
        assert_eq!(new_subscription, Some(subscription));
    }

    #[actix_rt::test]
    pub async fn create_subscription_already_existing() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Get existing subscription for comparison
        let existing_subscription =
            fetch_subscription_by_user_and_object(1, 1565, &mut *transaction)
                .await
                .unwrap()
                .unwrap();

        let subscription = Subscription {
            object_id: 1565,
            user_id: 1,
            send_email: false,
        };
        subscription.save(&mut *transaction).await.unwrap();

        // Verify that subscription was changed
        let subscription = fetch_subscription_by_user_and_object(1, 1565, &mut *transaction)
            .await
            .unwrap();
        assert_eq!(
            subscription,
            Some(Subscription {
                send_email: false,
                ..existing_subscription
            })
        )
    }

    #[actix_rt::test]
    async fn remove_subscription() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Verify assumption that the user is subscribed to object
        let existing_subscription =
            fetch_subscription_by_user_and_object(1, 1565, &mut *transaction)
                .await
                .unwrap();
        assert!(existing_subscription.is_some());

        let subscription = Subscription {
            object_id: 1565,
            user_id: 1,
            send_email: false,
        };
        subscription.remove(&mut *transaction).await.unwrap();

        // Verify that user is no longer subscribed to object
        let subscription = fetch_subscription_by_user_and_object(1, 1565, &mut *transaction)
            .await
            .unwrap();
        assert!(subscription.is_none());
    }

    pub async fn fetch_subscription_by_user_and_object<
        'a,
        A: sqlx::Acquire<'a, Database = sqlx::MySql>,
    >(
        user_id: i32,
        object_id: i32,
        acquire_from: A,
    ) -> Result<Option<Subscription>, sqlx::Error> {
        let subscriptions = Subscriptions::fetch_by_object(object_id, acquire_from).await?;
        let subscription = subscriptions
            .0
            .into_iter()
            .find(|subscription| subscription.user_id == user_id);
        Ok(subscription)
    }
}
