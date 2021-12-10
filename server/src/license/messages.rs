use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{fetch, fetch_via_transaction};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum LicenseMessage {
    LicenseQuery(license_query::Payload),
}

#[async_trait]
impl MessageResponder for LicenseMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            LicenseMessage::LicenseQuery(payload) => {
                payload.handle("LicenseQuery", connection).await
            }
        }
    }
}

pub mod license_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub id: i32,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub id: i32,
        pub instance: Instance,
        pub default: bool,
        pub title: String,
        pub url: String,
        pub content: String,
        pub agreement: String,
        pub icon_href: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => fetch(self.id, pool).await?,
                Connection::Transaction(transaction) => {
                    fetch_via_transaction(self.id, transaction).await?
                }
            })
        }
    }
}
