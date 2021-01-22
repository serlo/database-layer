use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscriptions {
    pub user_id: i32,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Error, Debug)]
pub enum SubscriptionsError {
    #[error("Subscriptions cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

#[derive(Serialize)]
pub struct Subscription {
    pub id: i32,
}

impl Subscriptions {
    pub async fn fetch(
        user_id: i32,
        pool: &MySqlPool,
    ) -> Result<Subscriptions, SubscriptionsError> {
        let subscriptions = sqlx::query!(
            r#"SELECT uuid_id FROM subscription WHERE user_id = ?"#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|inner| SubscriptionsError::DatabaseError { inner })?;

        let subscriptions: Vec<Subscription> = subscriptions
            .iter()
            .map(|child| Subscription {
                id: child.uuid_id as i32,
            })
            .collect();

        Ok(Subscriptions {
            user_id,
            subscriptions,
        })
    }
}
