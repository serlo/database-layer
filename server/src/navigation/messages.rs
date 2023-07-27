use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::Navigation;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum NavigationMessage {
    NavigationQuery(navigation_query::Payload),
}

#[async_trait]
impl MessageResponder for NavigationMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        self.handle(acquire_from).await
    }
}

pub mod navigation_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub instance: Instance,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Navigation;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let instance = self.instance.clone();
            Ok(Navigation::fetch(instance, acquire_from).await?)
        }
    }
}
