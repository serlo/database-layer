use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{
    Subscription, SubscriptionChangeError, SubscriptionChangePayload, SubscriptionsByUser,
    SubscriptionsError,
};
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubscriptionMessage {
    SubscriptionsQuery(SubscriptionsQuery),
    SubscriptionChangeMutation(SubscriptionChangeMutation),
}

#[async_trait]
impl MessageResponder for SubscriptionMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            SubscriptionMessage::SubscriptionsQuery(message) => message.handle(pool).await,
            SubscriptionMessage::SubscriptionChangeMutation(message) => message.handle(pool).await,
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionChangeMutation {
    pub ids: Vec<i32>,
    pub user_id: i32,
    pub subscribe: bool,
    pub send_email: bool,
}

#[async_trait]
impl MessageResponder for SubscriptionChangeMutation {
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
                println!("/subscription/change-subscription: {:?}", e); //remove fake path when we move to messages completely
                match e {
                    SubscriptionChangeError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().json(None::<String>)
                    }
                }
            }
        }
    }
}
