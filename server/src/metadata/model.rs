use crate::database::Executor;
use serde::Serialize;
use serde_json::json;

use super::messages::entities_metadata_query::Payload;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityMetadata {
    #[serde(rename = "@context")]
    context: serde_json::Value,
    id: String,
    identifier: serde_json::Value,
    #[serde(rename = "type")]
    schema_type: Vec<String>,
    learning_resource_type: String,
    name: Option<String>,
    description: Option<String>,
    date_created: String,
    date_modified: String,
    license: serde_json::Value,
    version: String,
}

impl EntityMetadata {
    pub async fn find_all<'a, E>(
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
                    AND entity.type_id IN (48, 3, 7, 1, 4, 6)
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
            .map(|result| EntityMetadata {
                context: json!([
                    "https://w3id.org/kim/lrmi-profile/draft/context.jsonld",
                    { "@language": result.instance }
                ]),
                // TODO: Sollte "http" genutzt werden?!
                id: get_iri(result.id as i32),
                identifier: json!({
                    "type": "PropertyValue",
                    "propertyID": "UUID",
                    "value": result.id as i32,
                }),
                schema_type: vec![
                    "LearningResource".to_string(),
                    get_learning_resource_type(&result.resource_type)
                ],
                learning_resource_type: get_learning_resource_type(&result.resource_type),
                name: result.params.as_ref()
                    .and_then(|params| params.get("title"))
                    .and_then(|title| title.as_str())
                    .map(|title| title.to_string()),
                description: result.params.as_ref()
                    .and_then(|params| params.get("meta_description"))
                    .and_then(|title| title.as_str())
                    .map(|title| title.to_string()),
                date_created: result.date_created.to_rfc3339(),
                date_modified: result.date_modified.to_rfc3339(),
                license: json!({"id": result.license_url}),
                version: get_iri(result.version.unwrap())
            })
            .collect()
        )
    }
}

fn get_iri(id: i32) -> String {
    format!("https://serlo.org/{}", id).to_string()
}

fn get_learning_resource_type(entity_type: &String) -> String {
    match entity_type.as_str() {
        "article" | "course-page" => "Article",
        "course" => "Course",
        "text-exercise-group" | "text-exercise" => "Quiz",
        "video" => "Video",
        _ => "",
    }
    .to_string()
}
