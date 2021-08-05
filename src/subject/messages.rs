use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{Subjects, SubjectsError};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubjectsMessage {
    SubjectsQuery(SubjectsQuery),
}

#[async_trait]
impl MessageResponder for SubjectsMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            SubjectsMessage::SubjectsQuery(message) => message.handle(connection).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectsQuery {
    pub instance: Instance,
}

#[async_trait]
impl MessageResponder for SubjectsQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let instance = self.instance.clone();
        let subjects = match connection {
            Connection::Pool(pool) => Subjects::fetch(instance, pool).await,
            Connection::Transaction(transaction) => {
                Subjects::fetch_via_transaction(instance, transaction).await
            }
        };
        match subjects {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/subjects/{:?}: {:?}", self.instance, e);
                match e {
                    SubjectsError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    SubjectsError::InvalidInstance => HttpResponse::InternalServerError().finish(),
                }
            }
        }
    }
}
