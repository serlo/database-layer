use crate::database::Executor;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

pub struct Subscriptions(pub Vec<Subscription>);

#[derive(Error, Debug)]
pub enum SubscriptionsError {
    #[error("Subscriptions cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
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
        .await
        .map_err(|inner| SubscriptionsError::DatabaseError { inner })?;

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
        .await
        .map_err(|inner| SubscriptionsError::DatabaseError { inner })?;

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
