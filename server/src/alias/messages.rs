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
pub enum AliasMessage {
    AliasQuery(alias_query::Payload),
}

#[async_trait]
impl MessageResponder for AliasMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(&self, acquire_from: A,) -> HttpResponse {
        match self {
            AliasMessage::AliasQuery(payload) => payload.handle("AliasQuery", connection).await,
        }
    }
}

pub mod alias_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub instance: Instance,
        pub path: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub id: i32,
        pub instance: Instance,
        pub path: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        #[allow(clippy::async_yields_async)]
        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(&self,acquire_from: A,) -> operation::Result<Self::Output> {
            let path = self.path.as_str();
            let instance = self.instance.clone();
            Ok(match connection {
                Connection::Pool(pool) => fetch(path, instance, pool).await?,
                Connection::Transaction(transaction) => {
                    fetch_via_transaction(path, instance, transaction).await?
                }
            })
        }
    }
}
