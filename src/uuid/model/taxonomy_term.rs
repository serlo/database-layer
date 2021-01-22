use convert_case::{Case, Casing};
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::UuidError;
use crate::format_alias;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyTerm {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    #[serde(rename(serialize = "type"))]
    pub term_type: String,
    pub instance: String,
    pub name: String,
    pub description: Option<String>,
    pub weight: i32,
    pub parent_id: Option<i32>,
    pub children_ids: Vec<i32>,
}

impl TaxonomyTerm {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<TaxonomyTerm, UuidError> {
        let taxonomy_term = sqlx::query!(
            r#"
                SELECT u.trashed, term.name, type.name as term_type, instance.subdomain, term_taxonomy.description, term_taxonomy.weight, term_taxonomy.parent_id
                    FROM term_taxonomy
                    JOIN term ON term.id = term_taxonomy.term_id
                    JOIN taxonomy ON taxonomy.id = term_taxonomy.taxonomy_id
                    JOIN type ON type.id = taxonomy.type_id
                    JOIN instance ON instance.id = taxonomy.instance_id
                    JOIN uuid u ON u.id = term_taxonomy.id
                    WHERE term_taxonomy.id = ?
            "#,
            id
        )
        .fetch_one(pool)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => UuidError::NotFound,
                inner => UuidError::DatabaseError { inner },
            })?;

        let entities_fut = sqlx::query!(
            r#"
                SELECT entity_id
                    FROM term_taxonomy_entity
                    WHERE term_taxonomy_id = ?
                    ORDER BY position ASC
            "#,
            id
        )
        .fetch_all(pool);
        let children_fut = sqlx::query!(
            r#"
                SELECT id
                    FROM term_taxonomy
                    WHERE parent_id = ?
                    ORDER BY weight ASC
            "#,
            id
        )
        .fetch_all(pool);
        let subject_fut = TaxonomyTerm::fetch_canonical_subject(id, pool);
        let (entities, children, subject) = try_join!(entities_fut, children_fut, subject_fut)
            .map_err(|inner| UuidError::DatabaseError { inner })?;
        let mut children_ids: Vec<i32> = entities
            .iter()
            .map(|child| child.entity_id as i32)
            .collect();
        children_ids.extend(children.iter().map(|child| child.id as i32));
        Ok(TaxonomyTerm {
            __typename: "TaxonomyTerm".to_string(),
            id,
            trashed: taxonomy_term.trashed != 0,
            alias: format_alias(subject.as_deref(), id, Some(&taxonomy_term.name)),
            term_type: normalize_type(taxonomy_term.term_type.as_str()),
            instance: taxonomy_term.subdomain,
            name: taxonomy_term.name,
            description: taxonomy_term.description,
            weight: taxonomy_term.weight.unwrap_or(0),
            parent_id: taxonomy_term.parent_id.map(|id| id as i32),
            children_ids,
        })
    }

    pub async fn fetch_canonical_subject(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<String>, sqlx::Error> {
        // Yes, this is super hacky. Didn't find a better way to handle recursion in MySQL 5 (in production, the max depth is around 10 at the moment)
        let subjects = sqlx::query!(
            r#"
                SELECT t.name
                    FROM term_taxonomy t0
                    LEFT JOIN term_taxonomy t1 ON t1.parent_id = t0.id
                    LEFT JOIN term_taxonomy t2 ON t2.parent_id = t1.id
                    LEFT JOIN term_taxonomy t3 ON t3.parent_id = t2.id
                    LEFT JOIN term_taxonomy t4 ON t4.parent_id = t3.id
                    LEFT JOIN term_taxonomy t5 ON t5.parent_id = t4.id
                    LEFT JOIN term_taxonomy t6 ON t6.parent_id = t5.id
                    LEFT JOIN term_taxonomy t7 ON t7.parent_id = t6.id
                    LEFT JOIN term_taxonomy t8 ON t8.parent_id = t7.id
                    LEFT JOIN term_taxonomy t9 ON t9.parent_id = t8.id
                    LEFT JOIN term_taxonomy t10 ON t10.parent_id = t9.id
                    LEFT JOIN term_taxonomy t11 ON t11.parent_id = t10.id
                    LEFT JOIN term_taxonomy t12 ON t12.parent_id = t11.id
                    LEFT JOIN term_taxonomy t13 ON t13.parent_id = t12.id
                    LEFT JOIN term_taxonomy t14 ON t14.parent_id = t13.id
                    LEFT JOIN term_taxonomy t15 ON t15.parent_id = t14.id
                    LEFT JOIN term_taxonomy t16 ON t16.parent_id = t15.id
                    LEFT JOIN term_taxonomy t17 ON t17.parent_id = t16.id
                    LEFT JOIN term_taxonomy t18 ON t18.parent_id = t17.id
                    LEFT JOIN term_taxonomy t19 ON t19.parent_id = t18.id
                    LEFT JOIN term_taxonomy t20 ON t20.parent_id = t19.id
                    JOIN term t on t1.term_id = t.id
                    WHERE
                        t0.parent_id IS NULL AND
                        (
                            t1.id = ? OR t2.id = ? OR t3.id = ? OR t4.id = ? OR t5.id = ? OR t6.id = ? OR t7.id = ? OR t8.id = ? OR t9.id = ? OR t10.id = ? OR
                            t11.id = ? OR t12.id = ? OR t13.id = ? OR t14.id = ? OR t15.id = ? OR t16.id = ? OR t17.id = ? OR t18.id = ? OR t19.id = ? OR t20.id = ?
                        )
            "#,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id,
            id
        ).fetch_one(pool).await;
        match subjects {
            Ok(subject) => Ok(Some(subject.name)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(inner) => Err(inner),
        }
    }
}

fn normalize_type(typename: &str) -> String {
    typename.to_case(Case::Camel)
}
