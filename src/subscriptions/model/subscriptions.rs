use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

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
            user_id,
            subscriptions,
        })
    }
}
