use crate::operation::{self, Operation};
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{fetch_subscriptions_by_user, Subscription};
use crate::message::MessageResponder;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubscriptionMessage {
    SubscriptionsQuery(subscriptions_query::Payload),
    SubscriptionSetMutation(subscription_set_mutation::Payload),
}

#[async_trait]
impl MessageResponder for SubscriptionMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            SubscriptionMessage::SubscriptionsQuery(message) => {
                message
                    .handle(format!("{:?}", message).as_str(), acquire_from)
                    .await
            }
            SubscriptionMessage::SubscriptionSetMutation(message) => {
                message
                    .handle(format!("{:?}", message).as_str(), acquire_from)
                    .await
            }
        }
    }
}

pub mod subscriptions_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub subscriptions: Vec<SubscriptionByUser>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SubscriptionByUser {
        pub object_id: i32,
        pub send_email: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(fetch_subscriptions_by_user(self.user_id, acquire_from).await?)
        }
    }
}

pub mod subscription_set_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub ids: Vec<i32>,
        pub user_id: i32,
        pub subscribe: bool,
        pub send_email: bool,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;
        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Subscription::change_subscription(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}
