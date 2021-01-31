use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{Uuid, UuidError};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UuidMessage {
    UuidQuery(UuidQuery),
}

impl UuidMessage {
    pub async fn handle(&self, pool: &MySqlPool) -> impl Responder {
        match self {
            UuidMessage::UuidQuery(message) => message.handle(pool).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UuidQuery {
    pub id: i32,
}

impl UuidQuery {
    pub async fn handle(&self, pool: &MySqlPool) -> impl Responder {
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
