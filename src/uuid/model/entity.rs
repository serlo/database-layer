use anyhow::Result;
use convert_case::{Case, Casing};
use futures::try_join;
use serde::Serialize;
use serlo_org_database_layer::{format_alias, format_datetime};
use sqlx::MySqlPool;

use super::taxonomy_term::TaxonomyTerm;

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
                SELECT t.name, u.trashed, i.subdomain, e.date, e.current_revision_id, e.license_id, f1.value as title, f2.value as fallback_title
                    FROM entity e
                    JOIN uuid u ON u.id = e.id
                    JOIN instance i ON i.id = e.instance_id
                    JOIN type t ON t.id = e.type_id
                    LEFT JOIN entity_revision_field f1 ON f1.entity_revision_id = e.current_revision_id AND f1.field = 'title'
                    LEFT JOIN entity_revision_field f2 on f2.entity_revision_id = (SELECT id FROM entity_revision WHERE repository_id = ? LIMIT 1) AND f2.field = 'title'
                    WHERE e.id = ?
            "#,
            id,
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
        let subject_fut = Self::find_canonical_subject_by_id(id, pool);
        let (entity, revisions, taxonomy_terms, subject) =
            try_join!(entity_fut, revisions_fut, taxonomy_terms_fut, subject_fut)?;

        Ok(Entity {
            __typename: normalize_type(entity.name.as_str()),
            id,
            trashed: entity.trashed != 0,
            alias: format_alias(
                subject.as_deref(),
                id,
                Some(
                    entity
                        .title
                        .or(entity.fallback_title)
                        .unwrap_or(format!("{}", id))
                        .as_str(),
                ),
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
            parent_id: Self::find_parent_by_id(id, pool).await?,
            page_ids: if entity.name == "course" {
                Some(Self::find_children_by_id_and_type(id, "course-page", pool).await?)
            } else {
                None
            },
            exercise_ids: if entity.name == "text-exercise-group" {
                Some(Self::find_children_by_id_and_type(id, "grouped-text-exercise", pool).await?)
            } else {
                None
            },
            solution_id: if entity.name == "text-exercise" || entity.name == "grouped-text-exercise"
            {
                // This double-wrapping is intentional. So basically:
                // Entities that aren't exercises have no solutionId in the serialized json
                // Exercises have an optional solutionId in the serialized json (i.e. number | null).
                // Might not be absolutely necessary but this achieves full feature-parity with serlo.org/api/*
                Some(
                    Self::find_children_by_id_and_type(id, "text-solution", pool)
                        .await?
                        .first()
                        .cloned(),
                )
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
                        UNION ALL
                        SELECT t.term_taxonomy_id, l2.child_id as entity_id
                            FROM term_taxonomy_entity t
                            JOIN entity_link l1 ON t.entity_id = l1.parent_id
                            JOIN entity_link l2 ON l2.parent_id = l1.child_id
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

    async fn find_parent_by_id(id: i32, pool: &MySqlPool) -> Result<Option<i32>, sqlx::Error> {
        let parents = sqlx::query!(
            r#"
                SELECT l.parent_id as id
                    FROM entity_link l
                    WHERE l.child_id = ?
            "#,
            id
        )
        .fetch_all(pool)
        .await?;
        Ok(parents
            .iter()
            .map(|parent| parent.id as i32)
            .collect::<Vec<i32>>()
            .first()
            .cloned())
    }

    async fn find_children_by_id_and_type(
        id: i32,
        children_type: &str,
        pool: &MySqlPool,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let children = sqlx::query!(
            r#"
                SELECT c.id
                    FROM entity_link l
                    JOIN entity c on c.id = l.child_id
                    JOIN type t ON t.id = c.type_id
                    WHERE l.parent_id = ? AND t.name = ?
                    ORDER BY l.order ASC
            "#,
            id,
            children_type,
        )
        .fetch_all(pool)
        .await?;
        Ok(children.iter().map(|child| child.id as i32).collect())
    }
}

fn normalize_type(typename: &str) -> String {
    let typename = typename.replace("text-", "");
    typename.to_case(Case::Pascal)
}
