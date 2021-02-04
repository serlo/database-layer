use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{Alias, AliasError};
use crate::instance::Instance;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum AliasMessage {
    AliasQuery(AliasQuery),
}

#[async_trait]
impl MessageResponder for AliasMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            AliasMessage::AliasQuery(message) => message.handle(pool).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasQuery {
    pub instance: Instance,
    pub path: String,
}

#[async_trait]
impl MessageResponder for AliasQuery {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match Alias::fetch(self.path.as_str(), self.instance.clone(), pool).await {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!(
                    "/alias/{:?}/{}: {:?}",
                    self.instance.clone(),
                    self.path.clone(),
                    e
                );
                match e {
                    AliasError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    AliasError::InvalidInstance => HttpResponse::InternalServerError().finish(),
                    AliasError::LegacyRoute => HttpResponse::NotFound().json(None::<String>),
                    AliasError::NotFound => HttpResponse::NotFound().json(None::<String>),
                }
            }
        }
    }
}
