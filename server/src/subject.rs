use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SubjectsMessage {
    SubjectsQuery(Option<serde_json::Value>),
}

#[async_trait]
impl MessageResponder for SubjectsMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(&self, acquire_from: A,) -> HttpResponse {
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

    async fn fetch_subjects<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        acquire_from: A,
    ) -> Result<subjects_query::Output, sqlx::Error> {
        let mut connection = acquire_from.acquire().await?;
        let subjects = sqlx::query!(
            r#"
                SELECT
                    subject.id,
                    subject_instance.subdomain as instance
                FROM term_taxonomy AS subject
                JOIN term_taxonomy AS root ON root.id = subject.parent_id
                JOIN uuid as subject_uuid ON subject_uuid.id = subject.id
                JOIN taxonomy AS subject_taxonomy ON subject_taxonomy.id = subject.taxonomy_id
                JOIN type AS subject_type ON subject_type.id = subject_taxonomy.type_id
                JOIN term AS subject_term ON subject_term.id = subject.term_id
                JOIN instance AS subject_instance ON subject_instance.id = subject_term.instance_id
                WHERE
                    (root.parent_id IS NULL
                      OR root.id = 106081
                      OR root.id = 146728)
                    AND subject_uuid.trashed = 0
                    AND (subject_type.name = "subject" or subject_type.name = "topic")
                ORDER BY subject.id;

            "#,
        )
        .fetch_all(&mut *connection)
        .await?
        .iter()
        .map(|record| subjects_query::Subject {
            taxonomy_term_id: record.id as i32,
            instance: record.instance.clone(),
        })
        .collect();

        Ok(subjects_query::Output { subjects })
    }
}
