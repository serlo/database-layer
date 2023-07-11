use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::Navigation;
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum NavigationMessage {
    NavigationQuery(navigation_query::Payload),
}

#[async_trait]
impl MessageResponder for NavigationMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(&self, acquire_from: A,) -> HttpResponse {
        match self {
            NavigationMessage::NavigationQuery(message) => {
                message.handle("NavigationQuery", acquire_from).await
            }
        }
    }
}

pub mod navigation_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub instance: Instance,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Navigation;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(&self,acquire_from: A,) -> operation::Result<Self::Output> {
            let instance = self.instance.clone();
            Ok(match connection {
                Connection::Pool(pool) => Navigation::fetch(instance, pool).await?,
                Connection::Transaction(transaction) => {
                    Navigation::fetch_via_transaction(instance, transaction).await?
                }
            })
        }
    }
}
