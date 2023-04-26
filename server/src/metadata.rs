use actix_web::HttpResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::database::{Connection, Executor};
use crate::message::MessageResponder;
use crate::operation::Error;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum MetadataMessage {
    EntitiesMetadataQuery(entities_metadata_query::Payload),
}

#[async_trait]
impl MessageResponder for MetadataMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            MetadataMessage::EntitiesMetadataQuery(payload) => {
                payload.handle("EntitiesMetadataQuery", connection).await
            }
        }
    }
}

pub mod entities_metadata_query {
    use itertools::Itertools;
    use std::collections::HashSet;

    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub first: i32,
        pub after: Option<i32>,
        pub instance: Option<String>,
        pub modified_after: Option<DateTime<Utc>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        entities: Vec<EntityMetadata>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct EntityMetadata {
        #[serde(rename = "@context")]
        context: serde_json::Value,
        id: String,
        #[serde(rename = "type")]
        schema_type: Vec<String>,
        date_created: String,
        date_modified: String,
        description: Option<String>,
        headline: Option<String>,
        identifier: serde_json::Value,
        in_language: Vec<String>,
        is_accessible_for_free: bool,
        is_family_friendly: bool,
        is_part_of: Vec<LinkedNode>,
        learning_resource_type: Vec<LinkedNode>,
        license: LinkedNode,
        maintainer: String,
        name: String,
        publisher: Vec<LinkedNode>,
        version: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct LinkedNode {
        id: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            if self.first > 10_000 {
                return Err(Error::BadRequest {
                    reason: "The 'first' value should be less than or equal 10_000".to_string(),
                });
            };

            let entities = match connection {
                Connection::Pool(pool) => query(self, pool).await?,
                Connection::Transaction(transaction) => query(self, transaction).await?,
            };

            Ok(Output { entities })
        }
    }

    async fn query<'a, E>(
        payload: &Payload,
        executor: E,
    ) -> Result<Vec<EntityMetadata>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        Ok(sqlx::query!(
            r#"
                SELECT
                    entity.id,
                    type.name AS resource_type,
                    JSON_OBJECTAGG(entity_revision_field.field, entity_revision_field.value) AS params,
                    entity.date AS date_created,
                    entity_revision.date AS date_modified,
                    entity.current_revision_id AS version,
                    license.url AS license_url,
                    instance.subdomain AS instance,
                    JSON_ARRAYAGG(term_taxonomy.id) AS taxonomy_term_ids,
                    JSON_OBJECTAGG(term_taxonomy.id, term.name) AS term_names
                FROM entity
                JOIN uuid ON uuid.id = entity.id
                JOIN instance ON entity.instance_id = instance.id
                JOIN type on entity.type_id = type.id
                JOIN license on license.id = entity.license_id
                JOIN entity_revision ON entity.current_revision_id = entity_revision.id
                JOIN entity_revision_field on entity_revision_field.entity_revision_id = entity_revision.id
                JOIN term_taxonomy_entity on term_taxonomy_entity.entity_id = entity.id
                JOIN term_taxonomy on term_taxonomy_entity.term_taxonomy_id = term_taxonomy.id
                JOIN term on term_taxonomy.term_id = term.id
                WHERE entity.id > ?
                    AND (? is NULL OR instance.subdomain = ?)
                    AND (? is NULL OR entity_revision.date > ?)
                    AND uuid.trashed = 0
                    AND type.name IN ("applet", "article", "course", "text-exercise",
                                      "text-exercise-group", "video")
                GROUP BY entity.id
                ORDER BY entity.id
                LIMIT ?
            "#,
            payload.after.unwrap_or(0),
            payload.instance,
            payload.instance,
            payload.modified_after,
            payload.modified_after,
            payload.first
        ).fetch_all(executor)
            .await?
            .into_iter()
            .map(|result| {
                let identifier = result.id as i32;
                let title: Option<String> = result.params.as_ref()
                    .and_then(|params| params.get("title"))
                    .and_then(|title| title.as_str())
                    .map(|title| title.to_string());
                let id = get_iri(result.id as i32);
                let schema_type =
                        get_schema_type(&result.resource_type);
                let name = title.clone().unwrap_or_else(|| {
                    let schema_type_i18n = match result.instance.as_str() {
                        "de" => {
                            match result.resource_type.as_str() {
                                "article" => "Artikel",
                                "course" => "Kurs",
                                "text-exercise" => "Aufgabe",
                                "text-exercise-group" => "Aufgabengruppe",
                                "video" => "Video",
                                "applet" => "Applet",
                                _ => "Inhalt",
                            }
                        },
                        _ => {
                            match result.resource_type.as_str() {
                                "article" => "Article",
                                "course" => "Course",
                                "text-exercise" => "Exercise",
                                "text-exercise-group" => "Exercise group",
                                "video" => "Video",
                                "applet" => "Applet",
                                _ => "Content",
                            }
                        }
                    };
                    // Here we select the term name of the taxonomy term with the smallest ID
                    // assuming that this is the taxonomy term of the main taxonomy (hopefully)
                    let term_name = result.term_names
                        .and_then(|value| {
                            value.as_object().and_then(|map| {
                                map.keys()
                                    .filter_map(|key| key.parse::<i64>().ok())
                                    .min()
                                    .map(|key| key.to_string())
                                    .and_then(|key| map.get(&key))
                                    .and_then(|value| value.as_str())
                                    .map(|name| String::from(name))
                            })
                        })
                        // Since we have a left join on term_taxonomy_entity we whould never hit
                        // this case (and thus avoid entites not being in a taxonomy)
                        .unwrap_or("<unknown>".to_string());

                    format!("{schema_type_i18n}#{identifier} in \"{term_name}\"")
                });
                let is_part_of: Vec<LinkedNode> = result.taxonomy_term_ids.as_ref()
                    .and_then(|value| value.as_array())
                    .map(|ids| {
                        ids.into_iter()
                           .filter_map(|element| element.as_i64())
                           // Since the query returns the same taxonomy term id for each parameter
                           // in `entity_revision_field` we need to remove duplicates from the list
                           .collect::<HashSet<i64>>()
                           .into_iter()
                           .sorted()
                           .map(|id| LinkedNode { id: get_iri(id as i32) })
                           .collect()
                    })
                    .unwrap_or(Vec::new());
                let publisher_id = "https://serlo.org/".to_string();

                EntityMetadata {
                    context: json!([
                        "https://w3id.org/kim/lrmi-profile/draft/context.jsonld",
                        { "@language": result.instance }
                    ]),
                    schema_type: vec!["LearningResource".to_string(), schema_type],
                    description: result.params.as_ref()
                        .and_then(|params| params.get("meta_description"))
                        .and_then(|title| title.as_str())
                        .map(|title| title.to_string()),
                    date_created: result.date_created.to_rfc3339(),
                    date_modified: result.date_modified.to_rfc3339(),
                    headline: title,
                    id,
                    identifier: json!({
                        "type": "PropertyValue",
                        "propertyID": "UUID",
                        "value": identifier,
                    }),
                    in_language: vec![result.instance],
                    is_accessible_for_free: true,
                    is_family_friendly: true,
                    learning_resource_type: get_learning_resource_type(&result.resource_type),
                    license: LinkedNode { id: result.license_url},
                    maintainer: publisher_id.clone(),
                    name,
                    publisher: vec![ LinkedNode { id: publisher_id.clone() }],
                    is_part_of,
                    version: get_iri(result.version.unwrap())
                }
            })
            .collect()
        )
    }

    fn get_iri(id: i32) -> String {
        format!("https://serlo.org/{id}")
    }

    fn get_schema_type(entity_type: &str) -> String {
        match entity_type {
            "article" => "Article",
            "course" => "Course",
            "text-exercise-group" | "text-exercise" => "Quiz",
            "video" => "Video",
            "applet" => "WebApplication",
            _ => entity_type,
        }
        .to_string()
    }

    fn get_learning_resource_type(entity_type: &str) -> Vec<LinkedNode> {
        match entity_type {
            "article" => vec!["text", "worksheet", "course", "web_page", "wiki"],
            "course" => vec!["course", "exploration", "web_page", "wiki"],
            "text-exercise-group" | "text-exercise" => {
                vec!["drill_and_practice", "assessment", "web_page", "wiki"]
            }
            "video" => vec!["video", "audiovisual_medium"],
            "applet" => vec!["application", "demonstration"],
            _ => vec![],
        }
        .into_iter()
        .map(|vocab| {
            format!(
                "http://w3id.org/openeduhub/vocabs/learningResourceType/{}",
                vocab
            )
        })
        .map(|id| LinkedNode { id })
        .collect()
    }
}
