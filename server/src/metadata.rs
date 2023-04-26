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
        learning_resource_type: String,
        license: serde_json::Value,
        maintainer: String,
        name: String,
        publisher: serde_json::Value,
        version: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            if self.first >= 10_000 {
                return Err(Error::BadRequest {
                    reason: "The 'first' value should be less than 10_000".to_string(),
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
                    instance.subdomain AS instance
                FROM entity
                JOIN uuid ON uuid.id = entity.id
                JOIN instance ON entity.instance_id = instance.id
                JOIN type on entity.type_id = type.id
                JOIN license on license.id = entity.license_id
                JOIN entity_revision ON entity.current_revision_id = entity_revision.id
                JOIN entity_revision_field on entity_revision_field.entity_revision_id = entity_revision.id
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
                let title: Option<String> = result.params.as_ref()
                    .and_then(|params| params.get("title"))
                    .and_then(|title| title.as_str())
                    .map(|title| title.to_string());
                let id = get_iri(result.id as i32);
                let learning_resource_type = get_learning_resource_type(&result.resource_type);
                let name = title.clone().unwrap_or_else(|| format!("{learning_resource_type}: {id}"));

                EntityMetadata {
                    context: json!([
                        "https://w3id.org/kim/lrmi-profile/draft/context.jsonld",
                        { "@language": result.instance }
                    ]),
                    schema_type: vec![
                        "LearningResource".to_string(),
                        get_learning_resource_type(&result.resource_type)
                    ],
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
                        "value": result.id as i32,
                    }),
                    in_language: vec![result.instance],
                    is_accessible_for_free: true,
                    is_family_friendly: true,
                    learning_resource_type,
                    license: json!({"id": result.license_url}),
                    maintainer: "https://serlo.org/".to_string(),
                    name,
                    publisher: json!([
                        { "id": "https://serlo.org/".to_string() }
                    ]),
                    version: get_iri(result.version.unwrap())
                }
            })
            .collect()
        )
    }

    fn get_iri(id: i32) -> String {
        format!("https://serlo.org/{id}")
    }

    fn get_learning_resource_type(entity_type: &str) -> String {
        match entity_type {
            "article" | "course-page" => "Article",
            "course" => "Course",
            "text-exercise-group" | "text-exercise" => "Quiz",
            "video" => "Video",
            "applet" => "WebApplication",
            _ => entity_type,
        }
        .to_string()
    }
}
