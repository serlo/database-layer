use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{License, LicenseError};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum LicenseMessage {
    LicenseQuery(LicenseQuery),
}

#[async_trait]
impl MessageResponder for LicenseMessage {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            LicenseMessage::LicenseQuery(message) => message.handle(connection).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseQuery {
    pub id: i32,
}

#[async_trait]
impl MessageResponder for LicenseQuery {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let license = match connection {
            Connection::Pool(pool) => License::fetch(self.id, pool).await,
            Connection::Transaction(transaction) => {
                License::fetch_via_transaction(self.id, transaction).await
            }
        };
        match license {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/license/{}: {:?}", self.id, e);
                match e {
                    LicenseError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    LicenseError::InvalidInstance => HttpResponse::InternalServerError().finish(),
                    LicenseError::NotFound => HttpResponse::NotFound().json(&None::<String>),
                }
            }
        }
    }
}
