use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;
use crate::datetime::DateTime;

#[derive(Debug, Eq, PartialEq)]
pub struct Subscriptions(pub Vec<Subscription>);

#[derive(Error, Debug)]
pub enum SubscriptionsError {
    #[error("Subscriptions cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for SubscriptionsError {
    fn from(inner: sqlx::Error) -> Self {
        SubscriptionsError::DatabaseError { inner }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Subscription {
    pub object_id: i32,
    pub user_id: i32,
    pub send_email: bool,
}

impl Subscriptions {
    pub async fn fetch_by_user<'a, E>(user_id: i32, executor: E) -> Result<Self, SubscriptionsError>
    where
        E: Executor<'a>,
    {
        let subscriptions = sqlx::query!(
            r#"SELECT uuid_id, user_id, notify_mailman FROM subscription WHERE user_id = ?"#,
            user_id
        )
        .fetch_all(executor)
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

    pub async fn fetch_by_object<'a, E>(
        object_id: i32,
        executor: E,
    ) -> Result<Self, SubscriptionsError>
    where
        E: Executor<'a>,
    {
        let subscriptions = sqlx::query!(
            r#"SELECT uuid_id, user_id, notify_mailman FROM subscription WHERE uuid_id = ?"#,
            object_id
        )
        .fetch_all(executor)
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
    pub async fn save<'a, E>(&self, executor: E) -> Result<(), SubscriptionChangeError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
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
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn remove<'a, E>(&self, executor: E) -> Result<(), SubscriptionChangeError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        sqlx::query!(
            r#"DELETE FROM subscription WHERE uuid_id = ? AND user_id = ?"#,
            self.object_id,
            self.user_id,
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionsByUser {
    user_id: i32,
    subscriptions: Vec<SubscriptionByUser>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionByUser {
    id: i32,
    send_email: bool,
}

impl SubscriptionsByUser {
    pub async fn fetch(user_id: i32, pool: &MySqlPool) -> Result<Self, SubscriptionsError> {
        Self::fetch_via_transaction(user_id, pool).await
    }

    pub async fn fetch_via_transaction<'a, E>(
        user_id: i32,
        executor: E,
    ) -> Result<Self, SubscriptionsError>
    where
        E: Executor<'a>,
    {
        let subscriptions = Subscriptions::fetch_by_user(user_id, executor).await?;
        let subscriptions = subscriptions
            .0
            .iter()
            .map(|child| SubscriptionByUser {
                id: child.object_id,
                send_email: child.send_email,
            })
            .collect();

        Ok(SubscriptionsByUser {
            user_id,
            subscriptions,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionChangePayload {
    pub ids: Vec<i32>,
    pub user_id: i32,
    pub subscribe: bool,
    pub send_email: bool,
}

#[derive(Error, Debug)]
pub enum SubscriptionChangeError {
    #[error("Subscription cannot be changed because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for SubscriptionChangeError {
    fn from(inner: sqlx::Error) -> Self {
        SubscriptionChangeError::DatabaseError { inner }
    }
}

impl Subscription {
    pub async fn change_subscription<'a, E>(
        payload: SubscriptionChangePayload,
        executor: E,
    ) -> Result<(), SubscriptionChangeError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for id in payload.ids.into_iter() {
            let subscription = Subscription {
                object_id: id,
                user_id: payload.user_id,
                send_email: payload.send_email,
            };

            if payload.subscribe {
                subscription.save(&mut transaction).await?;
            } else {
                subscription.remove(&mut transaction).await?;
            }
        }
        transaction.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Subscription, Subscriptions, SubscriptionsError};
    use crate::create_database_pool;
    use crate::database::Executor;

    #[actix_rt::test]
    async fn create_subscription_new() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Verify assumption that the user is not subscribed to object
        let subscription = fetch_subscription_by_user_and_object(1, 1555, &mut transaction)
            .await
            .unwrap();
        assert!(subscription.is_none());

        let subscription = Subscription {
            object_id: 1555,
            user_id: 1,
            send_email: true,
        };
        subscription.save(&mut transaction).await.unwrap();

        // Verify that user is now subscribed to object
        let new_subscription = fetch_subscription_by_user_and_object(1, 1555, &mut transaction)
            .await
            .unwrap();
        assert_eq!(new_subscription, Some(subscription));
    }

    #[actix_rt::test]
    async fn create_subscription_already_existing() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Get existing subscription for comparison
        let existing_subscription =
            fetch_subscription_by_user_and_object(1, 1565, &mut transaction)
                .await
                .unwrap()
                .unwrap();

        let subscription = Subscription {
            object_id: 1565,
            user_id: 1,
            send_email: false,
        };
        subscription.save(&mut transaction).await.unwrap();

        // Verify that subscription was changed
        let subscription = fetch_subscription_by_user_and_object(1, 1565, &mut transaction)
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
            fetch_subscription_by_user_and_object(1, 1565, &mut transaction)
                .await
                .unwrap();
        assert!(existing_subscription.is_some());

        let subscription = Subscription {
            object_id: 1565,
            user_id: 1,
            send_email: false,
        };
        subscription.remove(&mut transaction).await.unwrap();

        // Verify that user is no longer subscribed to object
        let subscription = fetch_subscription_by_user_and_object(1, 1565, &mut transaction)
            .await
            .unwrap();
        assert!(subscription.is_none());
    }

    async fn fetch_subscription_by_user_and_object<'a, E>(
        user_id: i32,
        object_id: i32,
        executor: E,
    ) -> Result<Option<Subscription>, SubscriptionsError>
    where
        E: Executor<'a>,
    {
        let subscriptions = Subscriptions::fetch_by_object(object_id, executor).await?;
        let subscription = subscriptions
            .0
            .into_iter()
            .find(|subscription| subscription.user_id == user_id);
        Ok(subscription)
    }
}
