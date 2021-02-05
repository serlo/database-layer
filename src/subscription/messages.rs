use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{SubscriptionsByUser, SubscriptionsError};
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubscriptionMessage {
    SubscriptionsQuery(SubscriptionsQuery),
}

#[async_trait]
impl MessageResponder for SubscriptionMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            SubscriptionMessage::SubscriptionsQuery(message) => message.handle(pool).await,
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
