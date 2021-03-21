use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{Navigation, NavigationError};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum NavigationMessage {
    NavigationQuery(NavigationQuery),
}

#[async_trait]
impl MessageResponder for NavigationMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            NavigationMessage::NavigationQuery(message) => message.handle(connection).await,
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
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let instance = self.instance.clone();
        let navigation = match connection {
            Connection::Pool(pool) => Navigation::fetch(instance, pool).await,
            Connection::Transaction(transaction) => {
                Navigation::fetch_via_transaction(instance, transaction).await
            }
        };
        match navigation {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
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
