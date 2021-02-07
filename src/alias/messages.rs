use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{Alias, AliasError};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::{MessageResponder, MessageResponderNew};

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

#[async_trait]
impl MessageResponderNew for AliasMessage {
    async fn handle_new(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            AliasMessage::AliasQuery(message) => message.handle_new(connection).await,
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
                println!("/alias/{:?}/{}: {:?}", self.instance, self.path, e);
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

#[async_trait]
impl MessageResponderNew for AliasQuery {
    async fn handle_new(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let path = self.path.as_str();
        let instance = self.instance.clone();
        let alias = match connection {
            Connection::Pool(pool) => Alias::fetch(path, instance, pool).await,
            Connection::Transaction(transaction) => {
                Alias::fetch_via_transaction(path, instance, transaction).await
            }
        };
        match alias {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/alias/{:?}/{}: {:?}", self.instance, self.path, e);
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
