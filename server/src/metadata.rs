use actix_web::HttpResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

use crate::message::MessageResponder;
use crate::operation::Error;
use crate::operation::{self, Operation};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum MetadataMessage {
    EntitiesMetadataQuery(entities_metadata_query::Payload),
}

#[async_trait]
impl MessageResponder for MetadataMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            MetadataMessage::EntitiesMetadataQuery(payload) => payload.handle(acquire_from).await,
        }
    }
}

pub mod entities_metadata_query {
    use itertools::Itertools;
    use std::collections::{HashMap, HashSet};

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
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
        #[serde(skip_serializing_if = "Option::is_none")]
        about: Option<Vec<SubjectMetadata>>,
        #[serde(rename = "@context")]
        context: serde_json::Value,
        id: String,
        #[serde(rename = "type")]
        schema_type: Vec<String>,
        date_created: String,
        date_modified: String,
        // The authors of the resource
        creator: Vec<Creator>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headline: Option<String>,
        identifier: serde_json::Value,
        in_language: Vec<String>,
        is_accessible_for_free: bool,
        is_family_friendly: bool,
        is_part_of: Vec<LinkedNode>,
        learning_resource_type: Vec<LinkedNode>,
        license: LinkedNode,
        main_entity_of_page: serde_json::Value,
        maintainer: serde_json::Value,
        name: String,
        publisher: Vec<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<LinkedNode>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Creator {
        #[serde(rename = "type")]
        creator_type: CreatorType,
        id: String,
        name: String,
        affiliation: serde_json::Value,
    }

    #[derive(Serialize)]
    enum CreatorType {
        Person,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct LinkedNode {
        id: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            if self.first > 10_000 {
                return Err(Error::BadRequest {
                    reason: "The 'first' value should be less than or equal 10_000".to_string(),
                });
            };
            let entities = query(self, acquire_from).await?;
            Ok(Output { entities })
        }
    }

    async fn query<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &Payload,
        acquire_from: A,
    ) -> Result<Vec<EntityMetadata>, operation::Error> {
        // See https://github.com/serlo/private-issues-sso-metadata-wallet/issues/37
        let metadata_api_last_changes_date: DateTime<Utc> = DateTime::parse_from_rfc3339(
            &env::var("METADATA_API_LAST_CHANGES_DATE")
                .expect("METADATA_API_LAST_CHANGES_DATE is not set."),
        )
        .map_err(|_| operation::Error::InternalServerError {
            error: "Error while parsing METADATA_API_LAST_CHANGES_DATE".into(),
        })?
        .with_timezone(&Utc);

        let modified_after = if payload.modified_after > Some(metadata_api_last_changes_date) {
            payload.modified_after
        } else {
            Option::None
        };

        let mut connection = acquire_from.acquire().await?;
        Ok(sqlx::query!(
            r#"
                WITH RECURSIVE ancestors AS (
                    SELECT id AS root_id, parent_id, id AS origin_id, id AS subject_id
                    FROM term_taxonomy

                    UNION

                    SELECT tt.id, tt.parent_id,  a.origin_id, a.root_id
                    FROM term_taxonomy tt
                    JOIN ancestors a ON tt.id = a.parent_id
                )
                SELECT
                    entity.id,
                    JSON_ARRAYAGG(ancestors.subject_id) AS subject_ids,
                    type.name AS resource_type,
                    MIN(field_title.value) AS title,
                    MIN(field_description.value) AS description,
                    entity.date AS date_created,
                    entity_revision.date AS date_modified,
                    entity.current_revision_id AS version,
                    license.url AS license_url,
                    instance.subdomain AS instance,
                    JSON_ARRAYAGG(term_taxonomy.id) AS taxonomy_term_ids,
                    JSON_OBJECTAGG(term_taxonomy.id, term.name) AS term_names,
                    JSON_OBJECTAGG(user.id, user.username) AS authors,
                    JSON_OBJECTAGG(all_revisions_of_entity.id, user.id) AS author_edits
                FROM entity
                JOIN uuid ON uuid.id = entity.id
                JOIN instance ON entity.instance_id = instance.id
                JOIN type on entity.type_id = type.id
                JOIN license on license.id = entity.license_id
                JOIN entity_revision ON entity.current_revision_id = entity_revision.id
                LEFT JOIN entity_revision_field field_title on
                    field_title.entity_revision_id = entity_revision.id AND
                    field_title.field = "title"
                LEFT JOIN entity_revision_field field_description on
                    field_description.entity_revision_id = entity_revision.id AND
                    field_description.field = "meta_description"
                JOIN term_taxonomy_entity on term_taxonomy_entity.entity_id = entity.id
                JOIN term_taxonomy on term_taxonomy_entity.term_taxonomy_id = term_taxonomy.id
                JOIN term on term_taxonomy.term_id = term.id
                JOIN entity_revision all_revisions_of_entity ON all_revisions_of_entity.repository_id = entity.id
                JOIN user ON all_revisions_of_entity.author_id = user.id
                JOIN ancestors on ancestors.origin_id = term_taxonomy_entity.term_taxonomy_id
                WHERE entity.id > ?
                    AND (? is NULL OR instance.subdomain = ?)
                    AND (? is NULL OR entity_revision.date > ?)
                    AND uuid.trashed = 0
                    AND type.name IN ("applet", "article", "course", "text-exercise",
                                      "text-exercise-group", "video")
                    AND (ancestors.parent_id is NULL OR ancestors.root_id = 106081 OR ancestors.root_id = 146728)
                GROUP BY entity.id
                ORDER BY entity.id
                LIMIT ?
            "#,
            payload.after.unwrap_or(0),
            payload.instance,
            payload.instance,
            modified_after,
            modified_after,
            payload.first
        ).fetch_all(&mut *connection)
            .await?
            .into_iter()
            .map(|result| {
                let identifier = result.id as i32;
                let title: Option<String> = result.title;
                let id = get_iri(result.id as i32);

                let authors_map: HashMap<i32, String> = result.authors
                    .and_then(|x| serde_json::from_value(x).ok())
                    .unwrap_or_default();

                let edit_counts: HashMap<i32, usize> = result.author_edits
                    .as_ref()
                    .and_then(|edits| edits.as_object())
                    .map(|edits| edits.values()
                        .filter_map(|author_id| author_id.as_i64())
                        .fold(HashMap::new(), |mut acc, author_id| {
                            *acc.entry(author_id as i32).or_insert(0) += 1;
                            acc
                        })
                    )
                    .unwrap_or_default();

                let creators: Vec<Creator> = authors_map.iter()
                    .map(|(id, username)| (id, username, edit_counts.get(id).unwrap_or(&0)))
                    .sorted_by(|(id1, _, count1), (id2, _, count2)| {
                        count2.cmp(count1).then(id1.cmp(id2))
                    })
                    .map(|(id,username, _)| Creator {
                        creator_type: CreatorType::Person,
                        // Id is a url that links to our authors. It can look like
                        // the following
                        // https://serlo.org/user/:userId/:username
                        // or simplified as here
                        // https://serlo.org/:userId
                        id: get_iri(*id),
                        name: username.to_string(),
                        affiliation: get_serlo_organization_metadata()
                    })
                    .collect();
                let schema_type =
                        get_schema_type(&result.resource_type);
                let name = title.clone().unwrap_or_else(|| {
                    let schema_type_i18n = match result.instance.as_str() {
                        "de" => {
                            match result.resource_type.as_str() {
                                "article" => "Artikel",
                                "course" => "Kurs",
                                "text-exercise" | "text-exercise-group" => "Aufgabe",
                                "video" => "Video",
                                "applet" => "Applet",
                                _ => "Inhalt",
                            }
                        },
                        _ => {
                            match result.resource_type.as_str() {
                                "article" => "Article",
                                "course" => "Course",
                                "text-exercise" | "text-exercise-group" => "Exercise",
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
                                    .map(String::from)
                            })
                        })
                        // Since we have a left join on term_taxonomy_entity we whould never hit
                        // this case (and thus avoid entites not being in a taxonomy)
                        .unwrap_or("<unknown>".to_string());
                    let from_i18n = if result.instance == "de" { "aus" } else { "from" };

                    format!("{schema_type_i18n} {from_i18n} \"{term_name}\"")
                });
                let is_part_of: Vec<LinkedNode> = result.taxonomy_term_ids.as_ref()
                    .and_then(|value| value.as_array())
                    .map(|ids| {
                        ids.iter()
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
                let current_date = Utc::now().to_rfc3339();
                let subject_ids: Vec<i32> = result.subject_ids.as_ref()
                    .and_then(|value| value.as_array())
                    .map(|ids| {
                        ids.iter()
                            .filter_map(|element| element.as_i64())
                            .collect::<HashSet<i64>>()
                            .into_iter()
                            .map(|id| id as i32)
                            .collect()
                    })
                    .unwrap_or(Vec::new());
                let subject_metadata = Option::from(subject_ids.iter().flat_map(|id| {
                    map_serlo_subjects_to_amb_standard(*id)
                }).collect::<Vec<SubjectMetadata>>()).filter(|v| !v.is_empty());
                EntityMetadata {
                    about: subject_metadata,
                    context: json!([
                        "https://w3id.org/kim/amb/context.jsonld",
                        {
                            "@language": result.instance,
                            "@vocab": "http://schema.org/",
                            "type": "@type",
                            "id": "@id"
                        }
                    ]),
                    schema_type: vec!["LearningResource".to_string(), schema_type],
                    description: result.description.filter(|title| !title.is_empty()),
                    date_created: result.date_created.to_rfc3339(),
                    date_modified: result.date_modified.to_rfc3339(),
                    headline: title
                        .filter (|t| !t.is_empty()),
                    creator: creators,
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
                    license: LinkedNode { id: result.license_url },
                    main_entity_of_page: json!([
                        {
                            "id": "https://serlo.org/metadata",
                            "type": "WebAPI",
                            "provider": get_serlo_organization_metadata(),
                            "dateCreated": current_date,
                            "dateModified": current_date,
                        }
                    ]),
                    maintainer: get_serlo_organization_metadata(),
                    name,
                    publisher: vec![get_serlo_organization_metadata()],
                    is_part_of,
                    version: result.version.map(|version| LinkedNode { id: get_iri(version) })
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
            "video" => "VideoObject",
            "applet" => "WebApplication",
            _ => entity_type,
        }
        .to_string()
    }

    fn get_learning_resource_type(entity_type: &str) -> Vec<LinkedNode> {
        let mut resource_types_exercise = vec![
            "assessment",
            "drill_and_practice",
            "text",
            "web_page",
            "wiki",
            "open_activity",
            "teaching_module",
            "tool",
            "worksheet",
        ];

        match entity_type {
            "article" => vec![
                "text",
                "worksheet",
                "course",
                "web_page",
                "wiki",
                "demonstration",
                "image",
                "open_activity",
                "teaching_module",
                "tool",
            ],
            "course" => vec![
                "course",
                "exploration",
                "web_page",
                "wiki",
                "assessment",
                "demonstration",
                "drill_and_practice",
                "educational_game",
                "enquiry_oriented_activity",
                "experiment",
                "text",
                "open_activity",
                "teaching_module",
                "tool",
            ],
            "text-exercise" => resource_types_exercise,
            "text-exercise-group" => {
                resource_types_exercise.push("data");
                resource_types_exercise
            }
            "video" => vec![
                "video",
                "audiovisual_medium",
                "demonstration",
                "audio",
                "teaching_module",
                "tool",
            ],
            "applet" => vec![
                "application",
                "assessment",
                "demonstration",
                "drill_and_practice",
                "experiment",
                "exploration",
                "other_asset_type",
                "teaching_module",
                "tool",
                "wiki",
            ],
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

    fn get_serlo_organization_metadata() -> serde_json::Value {
        json!({
            "id": "https://serlo.org/organization",
            "type": "Organization",
            "name": "Serlo Education e.V."
        })
    }

    enum SchemeId {
        UniversitySubject,
        SchoolSubject,
    }

    impl SchemeId {
        fn to_scheme_string(&self) -> String {
            match *self {
                SchemeId::UniversitySubject => {
                    "https://w3id.org/kim/hochschulfaechersystematik/scheme".to_string()
                }
                SchemeId::SchoolSubject => "http://w3id.org/kim/schulfaecher/".to_string(),
            }
        }
        fn to_id_string(&self) -> String {
            match *self {
                SchemeId::UniversitySubject => {
                    "https://w3id.org/kim/hochschulfaechersystematik/n".to_string()
                }
                SchemeId::SchoolSubject => "http://w3id.org/kim/schulfaecher/s".to_string(),
            }
        }
    }

    impl From<RawSubjectMetadata> for SubjectMetadata {
        fn from(data: RawSubjectMetadata) -> Self {
            SubjectMetadata {
                r#type: "Concept".to_string(),
                id: data.in_scheme.to_id_string() + &data.id,
                in_scheme: Scheme {
                    id: data.in_scheme.to_scheme_string(),
                },
            }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SubjectMetadata {
        r#type: String,
        id: String,
        in_scheme: Scheme,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Scheme {
        id: String,
    }

    struct RawSubjectMetadata {
        id: String,
        in_scheme: SchemeId,
    }

    fn map_serlo_subjects_to_amb_standard(id: i32) -> Vec<SubjectMetadata> {
        match id {
            // Mathematik (Schule)
            5 | 23593 | 141587 | 169580 => vec![RawSubjectMetadata {
                id: "1017".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Nachhaltigkeit => Biologie, Ethik (Schule)
            17744 | 48416 | 242851 => vec![
                RawSubjectMetadata {
                    id: "1001".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
                RawSubjectMetadata {
                    id: "1008".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
            ],
            // Chemie (Schule)
            18230 => vec![RawSubjectMetadata {
                id: "1002".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Biologie (Schule)
            23362 => vec![RawSubjectMetadata {
                id: "1001".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Englisch (Schule)
            25979 | 107557 | 113127 => vec![RawSubjectMetadata {
                id: "1007".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Latein (Schule)
            33894 | 106085 => vec![RawSubjectMetadata {
                id: "1016".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Physik (Schule)
            41107 => vec![RawSubjectMetadata {
                id: "1022".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Informatik (Schule)
            47899 => vec![RawSubjectMetadata {
                id: "1013".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Politik => Politik, Sachunterricht (Schule)
            79159 | 107556 => vec![
                RawSubjectMetadata {
                    id: "1023".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
                RawSubjectMetadata {
                    id: "1028".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
            ],
            // Medienbildung => Medienbildung, Informatik (Schule)
            106083 => vec![
                RawSubjectMetadata {
                    id: "1046".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
                RawSubjectMetadata {
                    id: "1013".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
            ],
            // Geografie (Schule)
            106084 => vec![RawSubjectMetadata {
                id: "1010".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Psychologie (Schule)
            106086 => vec![RawSubjectMetadata {
                id: "1043".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Deutsch als Zweitsprache (Schule)
            112723 => vec![RawSubjectMetadata {
                id: "1006".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Geschichte (Schule)
            136362 | 140528 => vec![RawSubjectMetadata {
                id: "1011".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Wirtschaftskunde (Schule)
            137757 => vec![RawSubjectMetadata {
                id: "1033".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Musik (Schule)
            167849 | 48415 => vec![RawSubjectMetadata {
                id: "1020".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Spanisch (Schule)
            190109 => vec![RawSubjectMetadata {
                id: "1030".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Italienisch (Schule)
            198076 => vec![RawSubjectMetadata {
                id: "1014".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Religionen, deren Wahrnehmung und Vorurteile => Ethik, Geschichte (Schule)
            208736 => vec![
                RawSubjectMetadata {
                    id: "1008".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
                RawSubjectMetadata {
                    id: "1011".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
            ],
            // Deutsch (Schule)
            210462 => vec![RawSubjectMetadata {
                id: "1005".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Französisch (Schule)
            227992 => vec![RawSubjectMetadata {
                id: "1009".to_string(),
                in_scheme: SchemeId::SchoolSubject,
            }
            .into()],
            // Sex Education => Sexualerziehung, Biologie (Schule)
            78339 => vec![
                RawSubjectMetadata {
                    id: "1029".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
                RawSubjectMetadata {
                    id: "1001".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
            ],
            // Materialwissenschaft
            141607 => vec![RawSubjectMetadata {
                id: "294".to_string(),
                in_scheme: SchemeId::UniversitySubject,
            }
            .into()],
            // Grammatik => Asiatische Sprachen und Kulturen/Asienwissenschaften
            140527 => vec![RawSubjectMetadata {
                id: "187".to_string(),
                in_scheme: SchemeId::UniversitySubject,
            }
            .into()],
            // Kommunikation => Psychologie, Deutsch (Schule)
            173235 => vec![
                RawSubjectMetadata {
                    id: "1043".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
                RawSubjectMetadata {
                    id: "1005".to_string(),
                    in_scheme: SchemeId::SchoolSubject,
                }
                .into(),
            ],
            _ => vec![],
        }
    }
}
