use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{Uuid, UuidError, UuidFetcher};
use crate::database::Connection;
use crate::message::MessageResponder;
use crate::uuid::{SetUuidStateError, SetUuidStatePayload};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UuidMessage {
    UuidQuery(UuidQuery),
    UuidSetStateMutation(UuidSetStateMutation),
}

#[async_trait]
impl MessageResponder for UuidMessage {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            UuidMessage::UuidQuery(message) => message.handle(connection).await,
            UuidMessage::UuidSetStateMutation(message) => message.handle(connection).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UuidQuery {
    pub id: i32,
}

#[async_trait]
impl MessageResponder for UuidQuery {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let uuid = match connection {
            Connection::Pool(pool) => Uuid::fetch(self.id, pool).await,
            Connection::Transaction(transaction) => {
                Uuid::fetch_via_transaction(self.id, transaction).await
            }
        };
        match uuid {
            Ok(uuid) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(uuid),
            Err(e) => {
                println!("/uuid/{}: {:?}", self.id, e);
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UuidSetStateMutation {
    pub ids: Vec<i32>,
    pub user_id: i32,
    pub trashed: bool,
}

#[async_trait]
impl MessageResponder for UuidSetStateMutation {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = SetUuidStatePayload {
            ids: self.ids.clone(),
            user_id: self.user_id,
            trashed: self.trashed,
        };
        let response = match connection {
            Connection::Pool(pool) => Uuid::set_uuid_state(payload, pool).await,
            Connection::Transaction(transaction) => {
                Uuid::set_uuid_state(payload, transaction).await
            }
        };
        match response {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                println!("/set-uuid-state: {:?}", e);
                match e {
                    SetUuidStateError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    SetUuidStateError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}
