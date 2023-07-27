use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::message::MessageResponder;
use crate::operation::{self, Operation};

use super::model::{Vocabulary, VocabularyError};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum VocabularyMessage {
    VocabularyTaxonomyQuery(taxonomy_vocabulary_query::Payload),
}

#[async_trait]
impl MessageResponder for VocabularyMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            VocabularyMessage::VocabularyTaxonomyQuery(payload) => {
                payload
                    .handle(format!("{:?}", payload).as_str(), acquire_from)
                    .await
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Vocabulary::fetch_taxonomy_vocabulary(self.instance.clone(), acquire_from).await?)
        }
    }

    impl From<VocabularyError> for operation::Error {
        fn from(error: VocabularyError) -> Self {
            Self::InternalServerError {
                error: Box::new(error),
            }
        }
    }
}
