use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{
    Subscription, SubscriptionChangeError, SubscriptionChangePayload, SubscriptionsByUser,
    SubscriptionsError,
};
use crate::database::Connection;
use crate::message::{MessageResponder, MessageResponderNew};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubscriptionMessage {
    SubscriptionsQuery(SubscriptionsQuery),
    SubscriptionSetMutation(SubscriptionSetMutation),
}

#[async_trait]
impl MessageResponder for SubscriptionMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            SubscriptionMessage::SubscriptionsQuery(message) => message.handle(pool).await,
            SubscriptionMessage::SubscriptionSetMutation(message) => message.handle(pool).await,
        }
    }
}

#[async_trait]
impl MessageResponderNew for SubscriptionMessage {
    async fn handle_new(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            SubscriptionMessage::SubscriptionsQuery(message) => {
                message.handle_new(connection).await
            }
            SubscriptionMessage::SubscriptionSetMutation(message) => {
                message.handle_new(connection).await
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionsQuery {
    pub user_id: i32,
}

#[async_trait]
impl MessageResponder for SubscriptionsQuery {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match SubscriptionsByUser::fetch(self.user_id, pool).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/subscriptions/{}: {:?}", self.user_id, e);
                match e {
                    SubscriptionsError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}

#[async_trait]
impl MessageResponderNew for SubscriptionsQuery {
    async fn handle_new(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let subscriptions = match connection {
            Connection::Pool(pool) => SubscriptionsByUser::fetch(self.user_id, pool).await,
            Connection::Transaction(transaction) => {
                SubscriptionsByUser::fetch_via_transaction(self.user_id, transaction).await
            }
        };
        match subscriptions {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/subscriptions/{}: {:?}", self.user_id, e);
                match e {
                    SubscriptionsError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSetMutation {
    pub ids: Vec<i32>,
    pub user_id: i32,
    pub subscribe: bool,
    pub send_email: bool,
}

#[async_trait]
impl MessageResponder for SubscriptionSetMutation {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        let payload = SubscriptionChangePayload {
            ids: self.ids.clone(),
            user_id: self.user_id,
            subscribe: self.subscribe,
            send_email: self.send_email,
        };
        match Subscription::change_subscription(payload, pool).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                println!("{:?}: {:?}", self, e);
                match e {
                    SubscriptionChangeError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}

#[async_trait]
impl MessageResponderNew for SubscriptionSetMutation {
    async fn handle_new(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = SubscriptionChangePayload {
            ids: self.ids.clone(),
            user_id: self.user_id,
            subscribe: self.subscribe,
            send_email: self.send_email,
        };
        let response = match connection {
            Connection::Pool(pool) => Subscription::change_subscription(payload, pool).await,
            Connection::Transaction(transaction) => {
                Subscription::change_subscription(payload, transaction).await
            }
        };
        match response {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                println!("{:?}: {:?}", self, e);
                match e {
                    SubscriptionChangeError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}
