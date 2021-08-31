use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

use super::model::fetch_subjects;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubjectsMessage {
    SubjectsQuery(Option<serde_json::Value>),
}

#[async_trait]
impl MessageResponder for SubjectsMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            SubjectsMessage::SubjectsQuery(_) => {
                subjects_query::Payload {}
                    .handle("SubjectsQuery", connection)
                    .await
            }
        }
    }
}

pub mod subjects_query {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {}

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub subjects: Vec<Subject>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Subject {
        pub instance: String,
        pub taxonomy_term_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => fetch_subjects(pool).await?,
                Connection::Transaction(transaction) => fetch_subjects(transaction).await?,
            })
        }
    }
}
