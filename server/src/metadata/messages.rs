use actix_web::HttpResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::Error;
use crate::operation::{self, Operation};

use super::model::EntityMetadata;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum MetadataMessage {
    EntitiesMetadataQuery(entities_metadata_query::Payload),
}

#[async_trait]
impl MessageResponder for MetadataMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            MetadataMessage::EntitiesMetadataQuery(payload) => {
                payload.handle("SubjectsQuery", connection).await
            }
        }
    }
}

pub mod entities_metadata_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub first: i32,
        pub after: Option<i32>,
        pub instance: Option<String>,
        pub modified_after: Option<DateTime<Utc>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        entities: Vec<EntityMetadata>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            if self.first >= 10_000 {
                return Err(Error::BadRequest {
                    reason: "The 'first' value should be less than 10_000".to_string(),
                });
            };

            let entities = match connection {
                Connection::Pool(pool) => EntityMetadata::find_all(self, pool).await?,
                Connection::Transaction(transaction) => {
                    EntityMetadata::find_all(self, transaction).await?
                }
            };

            Ok(Output { entities })
        }
    }
}
