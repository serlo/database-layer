use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

use super::model::Vocabulary;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum VocabularyMessage {
    TaxonomyVocabularyQuery(taxonomy_vocabulary_query::Payload),
}

#[async_trait]
impl MessageResponder for VocabularyMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            VocabularyMessage::TaxonomyVocabularyQuery(payload) => {
                payload.handle("TaxonomyVocabularyQuery", connection).await
            }
        }
    }
}

pub mod taxonomy_vocabulary_query {
    use crate::instance::Instance;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        instance: Instance,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = String;

        // TODO: error handling
        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => {
                    match Vocabulary::fetch_taxonomy_vocabulary(self.instance.clone(), pool).await {
                        Ok(result) => Ok(result),
                        Err(error) => Err(operation::Error::InternalServerError {
                            error: Box::new(error),
                        }),
                    }
                }
                Connection::Transaction(transaction) => {
                    match Vocabulary::fetch_taxonomy_vocabulary(self.instance.clone(), transaction)
                        .await
                    {
                        Ok(result) => Ok(result),
                        Err(error) => Err(operation::Error::InternalServerError {
                            error: Box::new(error),
                        }),
                    }
                }
            }
        }
    }
}
