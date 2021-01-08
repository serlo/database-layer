use crate::uuid::model::taxonomy_term::TaxonomyTerm;
use anyhow::Result;
use convert_case::{Case, Casing};
use database_layer_actix::{format_alias, format_datetime};
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub instance: String,
    pub date: String,
    pub current_revision_id: Option<i32>,
    pub revision_ids: Vec<i32>,
    pub license_id: i32,
    pub taxonomy_term_ids: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_ids: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercise_ids: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solution_id: Option<Option<i32>>,
}

impl Entity {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Entity> {
        let entity_fut = sqlx::query!(
            r#"
                SELECT t.name, u.trashed, i.subdomain, e.date, e.current_revision_id, e.license_id, f.value
                    FROM entity e
                    JOIN uuid u ON u.id = e.id
                    JOIN instance i ON i.id = e.instance_id
                    JOIN type t ON t.id = e.type_id
                    LEFT JOIN entity_revision_field f ON f.entity_revision_id = e.current_revision_id AND f.field = 'title'
                    WHERE e.id = ?
            "#,
            id
        )
        .fetch_one(pool);
        let revisions_fut = sqlx::query!(
            r#"SELECT id FROM entity_revision WHERE repository_id = ?"#,
            id
        )
        .fetch_all(pool);
        let taxonomy_terms_fut = sqlx::query!(
            r#"SELECT term_taxonomy_id as id FROM term_taxonomy_entity WHERE entity_id = ?"#,
            id
        )
        .fetch_all(pool);
        let links_fut = sqlx::query!(
            r#"
                SELECT c.id as child_id, ct.name as child_type, p.id as parent_id, pt.name as parent_type
                    FROM entity_link l
                    JOIN entity p ON p.id = l.parent_id
                    JOIN type pt ON pt.id = p.type_id
                    JOIN entity c on c.id = l.child_id
                    JOIN type ct ON ct.id = c.type_id
                    WHERE c.id = ? OR p.id = ?
            "#,
            id,
            id
        ).fetch_all(pool);
        let subject_fut = Self::find_canonical_subject_by_id(id, pool);
        let (entity, revisions, taxonomy_terms, links, subject) = try_join!(
            entity_fut,
            revisions_fut,
            taxonomy_terms_fut,
            links_fut,
            subject_fut
        )?;

        let parents: Vec<i32> = links
            .iter()
            .filter_map(|link| {
                if link.child_id as i32 == id {
                    Some(link.parent_id as i32)
                } else {
                    None
                }
            })
            .collect();
        let children: Vec<i32> = links
            .iter()
            .filter_map(|link| {
                if link.parent_id as i32 == id {
                    Some(link.child_id as i32)
                } else {
                    None
                }
            })
            .collect();

        Ok(Entity {
            __typename: normalize_type(entity.name.as_str()),
            id,
            trashed: entity.trashed != 0,
            alias: format_alias(
                subject.as_deref(),
                id,
                Some(entity.value.unwrap_or(format!("{}", id)).as_str()),
            ),
            instance: entity.subdomain,
            date: format_datetime(&entity.date),
            current_revision_id: entity.current_revision_id,
            revision_ids: revisions
                .iter()
                .rev()
                .map(|revision| revision.id as i32)
                .collect(),
            license_id: entity.license_id,
            taxonomy_term_ids: taxonomy_terms.iter().map(|term| term.id as i32).collect(),
            parent_id: parents.first().cloned(),
            page_ids: if entity.name == "course" {
                Some(children.clone())
            } else {
                None
            },
            exercise_ids: if entity.name == "text-exercise-group" {
                Some(children.clone())
            } else {
                None
            },
            solution_id: if entity.name == "text-exercise" || entity.name == "grouped-text-exercise"
            {
                // This double-wrapping is intentional. So basically:
                // Entities that aren't exercises have no solutionId in the serialized json
                // Exercises have an optional solutionId in the serialized json (i.e. number | null).
                // Might not be absolutely necessary but this achieves full feature-parity with serlo.org/api/*
                Some(children.first().cloned())
            } else {
                None
            },
        })
    }

    pub async fn find_canonical_subject_by_id(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<String>, sqlx::Error> {
        let taxonomy_terms = sqlx::query!(
            r#"
            SELECT term_taxonomy_id as id
                FROM (
                    SELECT term_taxonomy_id, entity_id FROM term_taxonomy_entity
                    UNION ALL
                    SELECT t.term_taxonomy_id, l.child_id as entity_id
                        FROM term_taxonomy_entity t
                        JOIN entity_link l ON t.entity_id = l.parent_id
                ) u
                WHERE entity_id = ?
            "#,
            id
        )
        .fetch_all(pool)
        .await?;
        let subject = match taxonomy_terms.first() {
            Some(term) => TaxonomyTerm::find_canonical_subject_by_id(term.id as i32, pool).await?,
            _ => None,
        };
        Ok(subject)
    }
}

fn normalize_type(typename: &str) -> String {
    let typename = typename.replace("text-", "");
    typename.to_case(Case::Pascal)
}
