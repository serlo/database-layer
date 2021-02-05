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
    pub async fn fetch_by_user(user_id: i32, pool: &MySqlPool) -> Result<Self, SubscriptionsError> {
        let subscriptions = sqlx::query!(
            r#"SELECT uuid_id, user_id, notify_mailman FROM subscription WHERE user_id = ?"#,
            user_id
        )
        .fetch_all(pool)
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

    pub async fn fetch_by_object(
        object_id: i32,
        pool: &MySqlPool,
    ) -> Result<Self, SubscriptionsError> {
        Self::fetch_by_object_via_transaction(object_id, pool).await
    }

    pub async fn fetch_by_object_via_transaction<'a, E>(
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
pub struct SubscriptionByUser {
    id: i32,
}

impl SubscriptionsByUser {
    pub async fn fetch(user_id: i32, pool: &MySqlPool) -> Result<Self, SubscriptionsError> {
        let subscriptions = Subscriptions::fetch_by_user(user_id, pool).await?;
        let subscriptions = subscriptions
            .0
            .iter()
            .map(|child| SubscriptionByUser {
                id: child.object_id,
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

            if payload.subscribe == true {
                subscription.save(&mut transaction).await?;
            } else {
                subscription.remove(&mut transaction).await?;
            }
        }
        transaction.commit().await?;

        Ok(())
    }
}

// TODO: add tests

#[cfg(test)]
mod tests {
    use crate::create_database_pool;
    use crate::subscription::Subscriptions;

    use super::Subscription;

    #[actix_rt::test]
    async fn create_subscription_new() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Verify assumption that the user is not subscribed to object (no subscriptions actually)
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(1555, &mut transaction)
            .await
            .unwrap();
        assert!(subscriptions.0.is_empty());

        let subscription = Subscription {
            object_id: 1555,
            user_id: 1,
            send_email: true,
        };
        subscription.save(&mut transaction).await.unwrap();

        // Verify that subscription was created.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(1555, &mut transaction)
            .await
            .unwrap();

        assert_eq!(subscriptions.0[0], subscription);
    }

    #[actix_rt::test]
    async fn create_subscription_already_existing() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // get existing results for comparison
        let existing_subscriptions =
            Subscriptions::fetch_by_object_via_transaction(1565, &mut transaction)
                .await
                .unwrap();

        let subscription = Subscription {
            object_id: 1565,
            user_id: 1,
            send_email: false,
        };
        subscription.save(&mut transaction).await.unwrap();

        // Verify that subscription was changed.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(1565, &mut transaction)
            .await
            .unwrap();

        let subscription = subscriptions
            .0
            .into_iter()
            .find(|subscription| subscription.user_id == 1)
            .unwrap();
        let existing_subscription = existing_subscriptions
            .0
            .into_iter()
            .find(|subscription| subscription.user_id == 1)
            .unwrap();

        assert_eq!(
            subscription,
            Subscription {
                send_email: false,
                ..existing_subscription
            }
        )
    }

    #[actix_rt::test]
    async fn remove_subscription() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // get existing results for comparison
        let existing_subscriptions =
            Subscriptions::fetch_by_object_via_transaction(1565, &mut transaction)
                .await
                .unwrap();

        let subscription = Subscription {
            object_id: 1565,
            user_id: 1,
            send_email: false,
        };
        subscription.remove(&mut transaction).await.unwrap();

        // Verify that subscription was changed.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(1565, &mut transaction)
            .await
            .unwrap();

        let existing_subscription = existing_subscriptions
            .0
            .into_iter()
            .find(|subscription| subscription.user_id == 1);

        assert!(existing_subscription.is_some());

        let subscription = subscriptions
            .0
            .into_iter()
            .find(|subscription| subscription.user_id == 1);

        assert!(subscription.is_none());
    }
}
