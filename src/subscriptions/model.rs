use serde::Serialize;
use sqlx::MySqlPool;
use std::hash::{Hash, Hasher};
use thiserror::Error;

pub struct Subscriptions;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionsByUser {
    user_id: i32,
    subscriptions: Vec<SubscriptionByUser>,
}

#[derive(Serialize)]
struct SubscriptionByUser {
    pub id: i32,
}

#[derive(Error, Debug)]
pub enum SubscriptionsError {
    #[error("Subscriptions cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl Subscriptions {
    pub async fn fetch_by_user(
        user_id: i32,
        pool: &MySqlPool,
    ) -> Result<SubscriptionsByUser, SubscriptionsError> {
        let subscriptions = sqlx::query!(
            r#"SELECT uuid_id FROM subscription WHERE user_id = ?"#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|inner| SubscriptionsError::DatabaseError { inner })?;

        let subscriptions: Vec<SubscriptionByUser> = subscriptions
            .iter()
            .map(|child| SubscriptionByUser {
                id: child.uuid_id as i32,
            })
            .collect();

        Ok(SubscriptionsByUser {
            user_id,
            subscriptions,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionsByObject {
    object_id: i32,
    pub subscriptions: Vec<SubscriptionByObject>,
}

#[derive(Serialize)]
pub struct SubscriptionByObject {
    pub user_id: i32,
}

impl Subscriptions {
    pub async fn fetch_by_object<'a, E>(
        id: i32,
        executor: E,
    ) -> Result<SubscriptionsByObject, SubscriptionsError> {
        // TODO: we probably should generalize `Subscriptions`.
        unimplemented!()
    }
}

impl PartialEq for SubscriptionByObject {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}
impl Eq for SubscriptionByObject {}
impl Hash for SubscriptionByObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
    }
}
