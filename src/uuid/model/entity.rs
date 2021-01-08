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
                    LEFT JOIN entity_revision_field f ON f.entity_revision_id = e.current_revision_id
                    WHERE e.id = ?
                        AND f.field = 'title'
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
        let (entity, revisions, taxonomy_terms) =
            try_join!(entity_fut, revisions_fut, taxonomy_terms_fut)?;
        let subject = match taxonomy_terms.first() {
            Some(term) => TaxonomyTerm::find_canonical_subject_by_id(term.id as i32, pool).await?,
            _ => None,
        };
        Ok(Entity {
            __typename: normalize_type(entity.name),
            id,
            trashed: entity.trashed != 0,
            alias: format_alias(subject.as_deref(), id, entity.value.as_deref()),
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
        })
    }
}

fn normalize_type(typename: String) -> String {
    let typename = typename.replace("text-", "");
    typename.to_case(Case::Pascal)
}
