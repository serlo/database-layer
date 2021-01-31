use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{Uuid, UuidError};
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UuidMessage {
    UuidQuery(UuidQuery),
}

#[async_trait]
impl MessageResponder for UuidMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            UuidMessage::UuidQuery(message) => message.handle(pool).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UuidQuery {
    pub id: i32,
}

#[async_trait]
impl MessageResponder for UuidQuery {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        let id = self.id;
        match Uuid::fetch(id, pool).await {
            Ok(uuid) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(uuid),
            Err(e) => {
                println!("/uuid/{}: {:?}", id, e);
                match e {
                    UuidError::DatabaseError { .. } => HttpResponse::InternalServerError().finish(),
                    UuidError::InvalidInstance => HttpResponse::InternalServerError().finish(),
                    UuidError::UnsupportedDiscriminator { .. } => {
                        HttpResponse::NotFound().json(None::<String>)
                    }
                    UuidError::UnsupportedEntityType { .. } => {
                        HttpResponse::NotFound().json(None::<String>)
                    }
                    UuidError::UnsupportedEntityRevisionType { .. } => {
                        HttpResponse::NotFound().json(None::<String>)
                    }
                    UuidError::EntityMissingRequiredParent => {
                        HttpResponse::NotFound().json(None::<String>)
                    }
                    UuidError::NotFound => HttpResponse::NotFound().json(None::<String>),
                }
            }
        }
    }
}
