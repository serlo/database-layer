use crate::operation::{self, Operation};
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{
    fetch_subscriptions_by_user, fetch_subscriptions_by_user_via_transaction, Subscription,
};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubscriptionMessage {
    SubscriptionsQuery(subscriptions_query::Payload),
    SubscriptionSetMutation(subscription_set_mutation::Payload),
}

#[async_trait]
impl MessageResponder for SubscriptionMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            SubscriptionMessage::SubscriptionsQuery(message) => {
                message.handle("SubscriptionsQuery", connection).await
            }
            SubscriptionMessage::SubscriptionSetMutation(message) => {
                message.handle("SubscriptionSetMutation", connection).await
            }
        }
    }
}

pub mod subscriptions_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => fetch_subscriptions_by_user(self.user_id, pool).await?,
                Connection::Transaction(transaction) => {
                    fetch_subscriptions_by_user_via_transaction(self.user_id, transaction).await?
                }
            })
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
        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => Subscription::change_subscription(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Subscription::change_subscription(self, transaction).await?
                }
            };
            Ok(Output { success: true })
        }
    }
}
