use crate::database::Executor;
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::database::Connection;
use crate::message::MessageResponder;

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
            SubjectsMessage::SubjectsQuery(_) => subjects_query(connection).await,
        }
    }
}

async fn subjects_query(connection: Connection<'_, '_>) -> HttpResponse {
    let subjects = match connection {
        Connection::Pool(pool) => fetch_subjects(pool).await,
        Connection::Transaction(transaction) => fetch_subjects(transaction).await,
    };
    match subjects {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(&data),
        Err(e) => {
            println!("/subjects: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Subjects {
    pub subjects: Vec<Subject>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Subject {
    instance: String,
    taxonomy_term_id: i32,
}

async fn fetch_subjects<'a, E>(executor: E) -> Result<Subjects, sqlx::Error>
where
    E: Executor<'a>,
{
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
                    (root.parent_id IS NULL OR root.id = 106081)
                    AND subject_uuid.trashed = 0
                    AND (subject_type.name = "subject" or subject_type.name = "topic")
                ORDER BY subject.id;

            "#,
    )
    .fetch_all(executor)
    .await?
    .iter()
    .map(|record| Subject {
        taxonomy_term_id: record.id as i32,
        instance: record.instance.clone(),
    })
    .collect();

    Ok(Subjects { subjects })
}
