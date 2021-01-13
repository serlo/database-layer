use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscriptions {
    pub user_id: i32,
    pub subscriptions: Vec<Subscription>,
}

#[derive(Serialize)]
pub struct Subscription {
    pub id: i32,
}

impl Subscriptions {
    pub async fn get_subscriptions_for_user(
        user_id: i32,
        pool: &MySqlPool,
    ) -> Result<Subscriptions> {
        sqlx::query!(
            r#"SELECT id FROM uuid WHERE discriminator = "user" AND id = ?"#,
            user_id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                anyhow::Error::new(SubscriptionsError::NotFound { user_id })
            }
            e => anyhow::Error::new(e),
        })?;

        let subscriptions_fut = sqlx::query!(
            "SELECT uuid_id FROM subscription WHERE user_id = ?",
            user_id
        )
        .fetch_all(pool)
        .await?;

        let subscriptions: Vec<Subscription> = subscriptions_fut
            .iter()
            .map(|child| Subscription {
                id: child.uuid_id as i32,
            })
            .collect();

        Ok(Subscriptions {
            user_id: user_id,
            subscriptions: subscriptions,
        })
    }
}

#[derive(Error, Debug)]
pub enum SubscriptionsError {
    #[error("Given id {user_id:?} is not a valid user id.")]
    NotFound { user_id: i32 },
}
