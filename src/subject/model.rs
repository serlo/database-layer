use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subjects {
    pub subjects: Vec<Subject>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    instance: String,
    taxonomy_term_id: i32,
}

#[derive(Error, Debug)]
pub enum SubjectsError {
    #[error("Subjects cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Subjects cannot be fetched because the instance is invalid.")]
    InvalidInstance,
}

impl From<sqlx::Error> for SubjectsError {
    fn from(inner: sqlx::Error) -> Self {
        SubjectsError::DatabaseError { inner }
    }
}

impl Subjects {
    pub async fn fetch(pool: &MySqlPool) -> Result<Subjects, SubjectsError> {
        Self::fetch_via_transaction(pool).await
    }

    pub async fn fetch_via_transaction<'a, E>(executor: E) -> Result<Subjects, SubjectsError>
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
}
