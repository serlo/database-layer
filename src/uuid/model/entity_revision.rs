use crate::uuid::model::taxonomy_term::TaxonomyTerm;
use anyhow::Result;
use convert_case::{Case, Casing};
use database_layer_actix::{format_alias, format_datetime};
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRevision {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub date: String,
    pub author_id: i32,
    pub repository_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub content: String,
    pub changes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<String>,
}

impl EntityRevision {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<EntityRevision> {
        let revision_fut = sqlx::query!(
            r#"
                SELECT t.name, u.trashed, r.date, r.author_id, r.repository_id
                    FROM entity_revision r
                    JOIN uuid u ON u.id = r.id
                    JOIN entity e ON e.id = r.repository_id
                    JOIN type t ON t.id = e.type_id
                    WHERE r.id = ?
            "#,
            id
        )
        .fetch_one(pool);
        let fields_fut = sqlx::query_as!(
            Field,
            r#"
                SELECT field, value
                    FROM entity_revision_field
                    WHERE entity_revision_id = ?
            "#,
            id
        )
        .fetch_all(pool);
        let taxonomy_terms_fut = sqlx::query!(
            r#"
                SELECT t.term_taxonomy_id as id
                    FROM entity_revision r
                    JOIN term_taxonomy_entity t ON t.entity_id = r.repository_id
                    WHERE r.id = ?
            "#,
            id
        )
        .fetch_all(pool);
        let (revision, fields, taxonomy_terms) =
            try_join!(revision_fut, fields_fut, taxonomy_terms_fut)?;
        let subject = match taxonomy_terms.first() {
            Some(term) => TaxonomyTerm::find_canonical_subject_by_id(term.id as i32, pool).await?,
            _ => None,
        };

        Ok(EntityRevision {
            __typename: format!("{}Revision", normalize_type(revision.name.as_str())),
            id,
            trashed: revision.trashed != 0,
            alias: format_alias(
                subject.as_deref(),
                id,
                Some(
                    get_field("title", &fields)
                        .unwrap_or(format!("{}", id))
                        .as_str(),
                ),
            ),
            date: format_datetime(&revision.date),
            author_id: revision.author_id as i32,
            repository_id: revision.repository_id as i32,
            title: if revision.name == "text-exercise"
                || revision.name == "text-exercise-group"
                || revision.name == "grouped-text-exercise"
                || revision.name == "solution"
            {
                None
            } else {
                Some(get_field("title", &fields).unwrap_or_else(|| String::from("")))
            },
            content: if revision.name == "video" {
                get_field("description", &fields).unwrap_or_else(|| String::from(""))
            } else {
                get_field("content", &fields).unwrap_or_else(|| String::from(""))
            },
            changes: get_field("changes", &fields).unwrap_or_else(|| String::from("")),
            meta_title: if revision.name == "applet"
                || revision.name == "article"
                || revision.name == "event"
            {
                Some(get_field("meta_title", &fields).unwrap_or_else(|| String::from("")))
            } else {
                None
            },
            meta_description: if revision.name == "applet"
                || revision.name == "article"
                || revision.name == "course"
                || revision.name == "event"
            {
                Some(get_field("meta_description", &fields).unwrap_or_else(|| String::from("")))
            } else {
                None
            },
            url: if revision.name == "video" {
                get_field("content", &fields)
            } else {
                get_field("url", &fields)
            },
        })
    }
}

fn normalize_type(typename: &str) -> String {
    let typename = typename.replace("text-", "");
    typename.to_case(Case::Pascal)
}

struct Field {
    field: String,
    value: String,
}

fn get_field(name: &str, fields: &[Field]) -> Option<String> {
    let matches = fields
        .iter()
        .filter(|field| field.field == name)
        .collect::<Vec<&Field>>();
    matches.first().map(|field| String::from(&field.value))
}
