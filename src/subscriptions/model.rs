use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;
use crate::datetime::DateTime;

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

    pub async fn create_subscription<'a, E>(
        object_id: i32,
        user_id: i32,
        send_email: bool,
        executor: E,
    ) -> Result<(), SubscriptionsError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        sqlx::query!(
            r#"
            INSERT INTO subscription (uuid_id, user_id, notify_mailman, date)
                SELECT ?, ?, ?, ?
                WHERE NOT EXISTS
                (SELECT id FROM subscription WHERE uuid_id = ? AND user_id = ?)
            "#,
            object_id,
            user_id,
            send_email,
            DateTime::now(),
            object_id,
            user_id,
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

#[cfg(test)]
mod tests {
    use crate::create_database_pool;
    use crate::subscriptions::Subscriptions;

    #[actix_rt::test]
    async fn create_subscription_new() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Verify assumption that the user is not subscribed to object (no subscriptions actually)
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(1555, &mut transaction)
            .await
            .unwrap();
        assert!(subscriptions.0.is_empty());

        Subscriptions::create_subscription(1555, 1, true, &mut transaction)
            .await
            .unwrap();

        // Verify that subscription was created.
        let subscriptions = Subscriptions::fetch_by_object_via_transaction(1555, &mut transaction)
            .await
            .unwrap();

        assert_eq!(subscriptions.0[0].object_id, 1555);
        assert_eq!(subscriptions.0[0].user_id, 1);
        assert_eq!(subscriptions.0[0].send_email, true);
    }

    #[actix_rt::test]
    async fn create_subscription_already_existing() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // get existing results for comparison
        let existing_subscriptions =
            sqlx::query!(r#"SELECT * FROM subscription WHERE uuid_id = ?"#, 1565)
                .fetch_all(&mut transaction)
                .await
                .unwrap();

        Subscriptions::create_subscription(1565, 1, true, &mut transaction)
            .await
            .unwrap();

        // Verify that subscription was not changed.
        let subscriptions = sqlx::query!(r#"SELECT * FROM subscription WHERE uuid_id = ?"#, 1565)
            .fetch_all(&mut transaction)
            .await
            .unwrap();

        assert_eq!(existing_subscriptions[0].id, subscriptions[0].id);
        assert_eq!(existing_subscriptions[0].date, subscriptions[0].date);
        assert_eq!(existing_subscriptions[1].id, subscriptions[1].id);
        assert_eq!(existing_subscriptions[1].date, subscriptions[1].date);
    }
}
