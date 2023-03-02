use crate::database::Executor;

use super::messages::subjects_query;

pub async fn fetch_subjects<'a, E>(executor: E) -> Result<subjects_query::Output, sqlx::Error>
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
                    (root.parent_id IS NULL
                      OR root.id = 106081
                      OR root.id = 268835)
                    AND subject_uuid.trashed = 0
                    AND (subject_type.name = "subject" or subject_type.name = "topic")
                ORDER BY subject.id;

            "#,
    )
    .fetch_all(executor)
    .await?
    .iter()
    .map(|record| subjects_query::Subject {
        taxonomy_term_id: record.id as i32,
        instance: record.instance.clone(),
    })
    .collect();

    Ok(subjects_query::Output { subjects })
}
