use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{Navigation, NavigationError};
use crate::instance::Instance;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum NavigationMessage {
    NavigationQuery(NavigationQuery),
}

#[async_trait]
impl MessageResponder for NavigationMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            NavigationMessage::NavigationQuery(message) => message.handle(pool).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationQuery {
    pub instance: Instance,
}

#[async_trait]
impl MessageResponder for NavigationQuery {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match Navigation::fetch(self.instance.clone(), pool).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/navigation/{:?}: {:?}", self.instance, e);
                match e {
                    NavigationError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}
