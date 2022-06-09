use std::collections::HashSet;

use async_trait::async_trait;
use convert_case::{Case, Casing};
use futures::join;
use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;
use sqlx::MySqlPool;

use super::{AssertExists, ConcreteUuid, Uuid, UuidError, UuidFetcher};

use crate::database::Executor;
use crate::event::{
    CreateTaxonomyLinkEventPayload, CreateTaxonomyTermEventPayload, RemoveTaxonomyLinkEventPayload,
    SetTaxonomyTermEventPayload,
};
use crate::instance::Instance;
use crate::uuid::model::taxonomy_term::messages::taxonomy_term_set_name_and_description_mutation;
use crate::uuid::Entity;
use crate::uuid::EntityType;
use crate::{format_alias, operation};
pub use messages::*;

mod messages;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaxonomyType {
    Root, // Level 0

    Blog, // below Root

    ForumCategory, // below Root or ForumCategory
    Forum,         // below ForumCategory

    Subject, // below Root

    Locale,                // below Subject or Locale
    Curriculum,            // below Locale
    CurriculumTopic,       // below Curriculum or CurriculumTopic
    CurriculumTopicFolder, // below CurriculumTopic

    Topic,       // below Subject or Topic
    TopicFolder, // below Topic
}

impl std::str::FromStr for TaxonomyType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}

impl sqlx::Type<MySql> for TaxonomyType {
    fn type_info() -> MySqlTypeInfo {
        str::type_info()
    }
}
impl<'q> sqlx::Encode<'q, MySql> for TaxonomyType {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let decoded = serde_json::to_value(self).unwrap();
        let decoded = decoded.as_str().unwrap();
        decoded.encode_by_ref(buf)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyTerm {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    #[serde(rename(serialize = "type"))]
    pub term_type: String,
    pub instance: Instance,
    pub name: String,
    pub description: Option<String>,
    pub weight: i32,
    pub parent_id: Option<i32>,
    pub children_ids: Vec<i32>,
    pub taxonomy_id: i32,
}

macro_rules! fetch_one_taxonomy_term {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT
                    u.trashed, term.name, type.name as term_type, instance.subdomain,
                    term_taxonomy.description, term_taxonomy.weight, term_taxonomy.parent_id,
                    term_taxonomy.taxonomy_id
                FROM term_taxonomy
                JOIN term ON term.id = term_taxonomy.term_id
                JOIN taxonomy ON taxonomy.id = term_taxonomy.taxonomy_id
                JOIN type ON type.id = taxonomy.type_id
                JOIN instance ON instance.id = taxonomy.instance_id
                JOIN uuid u ON u.id = term_taxonomy.id
                WHERE term_taxonomy.id = ?
            "#,
            $id
        )
        .fetch_one($executor)
    };
}

macro_rules! fetch_all_entities {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT entity_id
                    FROM term_taxonomy_entity
                    WHERE term_taxonomy_id = ?
                    ORDER BY position ASC
            "#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! fetch_all_children {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT id
                    FROM term_taxonomy
                    WHERE parent_id = ?
                    ORDER BY weight ASC
            "#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! fetch_subject {
    ($id: expr, $executor: expr) => {
        TaxonomyTerm::fetch_canonical_subject($id, $executor)
    };
}

macro_rules! to_taxonomy_term {
    ($id: expr, $taxonomy_term: expr, $entities: expr, $children: expr, $subject: expr) => {{
        let taxonomy_term = $taxonomy_term.map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let entities = $entities?;
        let children = $children?;
        let subject = $subject?;

        let mut children_ids: Vec<i32> = entities
            .iter()
            .map(|child| child.entity_id as i32)
            .collect();
        children_ids.extend(children.iter().map(|child| child.id as i32));
        Ok(Uuid {
            id: $id,
            trashed: taxonomy_term.trashed != 0,
            alias: format_alias(
                subject.map(|subject| subject.name).as_deref(),
                $id,
                Some(&taxonomy_term.name),
            ),
            concrete_uuid: ConcreteUuid::TaxonomyTerm(TaxonomyTerm {
                __typename: "TaxonomyTerm".to_string(),
                term_type: TaxonomyTerm::normalize_type(taxonomy_term.term_type.as_str()),
                instance: taxonomy_term
                    .subdomain
                    .parse()
                    .map_err(|_| UuidError::InvalidInstance)?,
                name: taxonomy_term.name,
                description: taxonomy_term.description,
                weight: taxonomy_term.weight.unwrap_or(0),
                parent_id: taxonomy_term.parent_id.map(|id| id as i32),
                children_ids,
                taxonomy_id: taxonomy_term.taxonomy_id as i32,
            }),
        })
    }};
}

#[async_trait]
impl UuidFetcher for TaxonomyTerm {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let taxonomy_term = fetch_one_taxonomy_term!(id, pool);
        let entities = fetch_all_entities!(id, pool);
        let children = fetch_all_children!(id, pool);
        let subject = fetch_subject!(id, pool);

        let (taxonomy_term, entities, children, subject) =
            join!(taxonomy_term, entities, children, subject);

        to_taxonomy_term!(id, taxonomy_term, entities, children, subject)
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let taxonomy_term = fetch_one_taxonomy_term!(id, &mut transaction).await;
        let entities = fetch_all_entities!(id, &mut transaction).await;
        let children = fetch_all_children!(id, &mut transaction).await;
        let subject = fetch_subject!(id, &mut transaction).await;

        transaction.commit().await?;

        to_taxonomy_term!(id, taxonomy_term, entities, children, subject)
    }
}

pub struct Subject {
    pub taxonomy_term_id: i32,
    pub name: String,
}

impl TaxonomyTerm {
    pub async fn fetch_canonical_subject<'a, E>(
        id: i32,
        executor: E,
    ) -> Result<Option<Subject>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        // Yes, this is super hacky. Didn't find a better way to handle recursion in MySQL 5 (in production, the max depth is around 10 at the moment)
        let subjects = sqlx::query!(
            r#"
                SELECT t.name as name, t1.id as id
                    FROM term_taxonomy t0
                    JOIN term_taxonomy t1 ON t1.parent_id = t0.id
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
        ).fetch_one(executor).await;
        match subjects {
            Ok(subject) => Ok(Some(Subject {
                taxonomy_term_id: subject.id as i32,
                name: subject.name,
            })),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(inner) => Err(inner),
        }
    }

    fn normalize_type(typename: &str) -> String {
        typename.to_case(Case::Camel)
    }

    pub async fn get_instance_id<'a, E>(
        term_taxonomy_id: i32,
        executor: E,
    ) -> Result<i32, operation::Error>
    where
        E: Executor<'a>,
    {
        Ok(sqlx::query!(
            r#"
                SELECT term.instance_id
                    FROM term_taxonomy
                    JOIN term
                        ON term.id = term_taxonomy.term_id
                    WHERE term_taxonomy.id = ?
            "#,
            term_taxonomy_id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: format!("Taxonomy term with id {} does not exist", term_taxonomy_id),
        })?
        .instance_id as i32)
    }

    pub async fn set_name_and_description<'a, E>(
        payload: &taxonomy_term_set_name_and_description_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let term = sqlx::query!(
            r#"
                SELECT term_id AS id, instance_id
                    FROM term_taxonomy
                    JOIN term
                    ON term.id = term_taxonomy.term_id
                    WHERE term_taxonomy.id = ?
            "#,
            payload.id
        )
        .fetch_optional(&mut transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: format!("Taxonomy term with id {} does not exist", payload.id),
        })?;

        sqlx::query!(
            r#"
                UPDATE term
                SET name = ?
                WHERE id = ?
            "#,
            payload.name,
            term.id,
        )
        .execute(&mut transaction)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(db_error) => {
                if db_error.message().contains("uq_term_name_language") {
                    return operation::Error::BadRequest {
                        reason: "Two taxonomy terms cannot have same name in same instance"
                            .to_string(),
                    };
                };
                operation::Error::InternalServerError {
                    error: Box::new(db_error),
                }
            }
            _ => operation::Error::InternalServerError {
                error: Box::new(error),
            },
        })?;

        sqlx::query!(
            r#"
                UPDATE term_taxonomy
                SET description = ?
                WHERE id = ?
            "#,
            payload.description,
            payload.id,
        )
        .execute(&mut transaction)
        .await?;

        SetTaxonomyTermEventPayload::new(payload.id, payload.user_id, term.instance_id)
            .save(&mut transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn create<'a, E>(
        payload: &taxonomy_term_create_mutation::Payload,
        executor: E,
    ) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator)
                    VALUES (0, "taxonomyTerm")
            "#,
        )
        .execute(&mut transaction)
        .await?;

        let taxonomy_term_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        let instance_id = Self::get_instance_id(payload.parent_id, &mut transaction).await?;

        let type_id = sqlx::query!(
            r#"
                SELECT type.id FROM type
                JOIN taxonomy
                    ON taxonomy.type_id = type.id
                WHERE type.name = ?
                    AND instance_id = ?
            "#,
            payload.taxonomy_type,
            instance_id
        )
        .fetch_one(&mut transaction)
        .await?
        .id as i32;

        sqlx::query!(
            r#"
                INSERT INTO taxonomy (type_id, instance_id)
                    VALUES (?, ?)
            "#,
            type_id,
            instance_id,
        )
        .execute(&mut transaction)
        .await?;

        let taxonomy_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        sqlx::query!(
            r#"
                INSERT INTO term (name, instance_id)
                    VALUES (?, ?)
            "#,
            payload.name,
            instance_id,
        )
        .execute(&mut transaction)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(db_error) => {
                if db_error.message().contains("uq_term_name_language") {
                    return operation::Error::BadRequest {
                        reason: "Two taxonomy terms cannot have same name in same instance"
                            .to_string(),
                    };
                };
                operation::Error::InternalServerError {
                    error: Box::new(db_error),
                }
            }
            _ => operation::Error::InternalServerError {
                error: Box::new(error),
            },
        })?;

        let term_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        sqlx::query!(
            r#"
                INSERT INTO term_taxonomy (id, term_id, taxonomy_id, parent_id, description)
                    VALUES (?, ?, ?, ?, ?)
            "#,
            taxonomy_term_id,
            term_id,
            taxonomy_id,
            payload.parent_id,
            payload.description,
        )
        .execute(&mut transaction)
        .await?;

        CreateTaxonomyTermEventPayload::new(taxonomy_term_id, payload.user_id, instance_id)
            .save(&mut transaction)
            .await?;

        let taxonomy_term = Self::fetch_via_transaction(taxonomy_term_id, &mut transaction).await?;

        transaction.commit().await?;

        Ok(taxonomy_term)
    }

    pub async fn create_entity_link<'a, E>(
        payload: &taxonomy_create_entity_links_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let instance_id = Self::get_instance_id(payload.taxonomy_term_id, &mut transaction).await?;

        for child_id in &payload.entity_ids {
            let entity_type = Entity::fetch_entity_type(*child_id, &mut transaction)
                .await?
                .ok_or(operation::Error::BadRequest {
                    reason: format!("entity with id {} does not exist", child_id),
                })?;

            match entity_type {
                EntityType::CoursePage | EntityType::GroupedExercise | EntityType::Solution => {
                    return Err(operation::Error::BadRequest {
                        reason: format!(
                            "entity with id {} cannot be linked to a taxonomy term",
                            child_id
                        ),
                    })
                }
                _ => (),
            };

            let is_child_already_linked_to_taxonomy = sqlx::query!(
                r#"
                    SELECT id FROM term_taxonomy_entity
                        WHERE entity_id = ?
                        AND term_taxonomy_id = ?
                "#,
                child_id,
                payload.taxonomy_term_id
            )
            .fetch_optional(&mut transaction)
            .await?;

            if is_child_already_linked_to_taxonomy.is_some() {
                continue;
            }

            let child_instance_id = sqlx::query!(
                r#"
                    SELECT instance_id
                        FROM entity
                        WHERE id = ?
                "#,
                child_id
            )
            .fetch_one(&mut transaction)
            .await?
            .instance_id as i32;

            if instance_id != child_instance_id {
                return Err(operation::Error::BadRequest {
                    reason: format!(
                        "Entity {} and taxonomy term {} are not in the same instance",
                        child_id, payload.taxonomy_term_id
                    ),
                });
            }

            let last_position = sqlx::query!(
                r#"
                    SELECT IFNULL(MAX(position), 0) AS current_last
                        FROM term_taxonomy_entity
                        WHERE term_taxonomy_id = ?
                "#,
                payload.taxonomy_term_id
            )
            .fetch_one(&mut transaction)
            .await?
            .current_last as i32
                + 1;

            sqlx::query!(
                r#"
                    INSERT INTO term_taxonomy_entity (entity_id, term_taxonomy_id, position)
                    VALUES (?, ?, ?)
                "#,
                child_id,
                payload.taxonomy_term_id,
                last_position
            )
            .execute(&mut transaction)
            .await?;

            CreateTaxonomyLinkEventPayload::new(
                *child_id,
                payload.taxonomy_term_id,
                payload.user_id,
                instance_id,
            )
            .save(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn delete_entity_link<'a, E>(
        payload: &taxonomy_delete_entity_links_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let instance_id = Self::get_instance_id(payload.taxonomy_term_id, &mut transaction).await?;

        for child_id in &payload.entity_ids {
            let term_taxonomy_entity_id = sqlx::query!(
                r#"
                    SELECT id FROM term_taxonomy_entity
                        WHERE entity_id = ?
                        AND term_taxonomy_id = ?
                "#,
                child_id,
                payload.taxonomy_term_id
            )
            .fetch_optional(&mut transaction)
            .await?
            .ok_or(operation::Error::BadRequest {
                reason: format!(
                    "Id {} is not linked to taxonomy term {}",
                    child_id, payload.taxonomy_term_id
                ),
            })?
            .id as i32;

            if 1 == sqlx::query!(
                r#"
                    SELECT count(*) AS quantity FROM term_taxonomy_entity
                        WHERE entity_id = ?
                "#,
                child_id,
            )
            .fetch_one(&mut transaction)
            .await?
            .quantity as i32
            {
                return Err(operation::Error::BadRequest {
                    reason: format!(
                        "Entity with id {} has to be linked to at least one taxonomy",
                        child_id
                    ),
                });
            };

            sqlx::query!(
                r#"DELETE FROM term_taxonomy_entity WHERE id = ?"#,
                term_taxonomy_entity_id,
            )
            .execute(&mut transaction)
            .await?;

            RemoveTaxonomyLinkEventPayload::new(
                *child_id,
                payload.taxonomy_term_id,
                payload.user_id,
                instance_id,
            )
            .save(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn sort<'a, E>(
        payload: &taxonomy_sort_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        Self::assert_exists(payload.taxonomy_term_id, &mut transaction).await?;

        let entities_ids: Vec<i32> =
            fetch_all_entities!(payload.taxonomy_term_id, &mut transaction)
                .await?
                .iter()
                .map(|child| child.entity_id as i32)
                .collect();

        let children_taxonomy_ids: Vec<i32> =
            fetch_all_children!(payload.taxonomy_term_id, &mut transaction)
                .await?
                .iter()
                .map(|child| child.id as i32)
                .collect();

        let mut children_ids: Vec<i32> = entities_ids.clone();
        children_ids.extend(children_taxonomy_ids.clone());

        if children_ids == payload.children_ids {
            return Ok(());
        }

        if HashSet::<i32>::from_iter(children_ids.clone())
            != HashSet::from_iter(payload.children_ids.clone())
        {
            return Err(operation::Error::BadRequest {
                reason: "children_ids have to match the current entities ids linked to the taxonomy_term_id".to_string()
            });
        }

        for (index, entity_id) in payload
            .children_ids
            .iter()
            .filter(|child_id| entities_ids.contains(child_id))
            .enumerate()
        {
            sqlx::query!(
                r#"
                    UPDATE term_taxonomy_entity
                    SET position = ?
                    WHERE term_taxonomy_id = ?
                        AND entity_id = ?
                "#,
                index as i32,
                payload.taxonomy_term_id,
                entity_id,
            )
            .execute(&mut transaction)
            .await?;
        }

        for (index, child_id) in payload
            .children_ids
            .iter()
            .filter(|child_id| children_taxonomy_ids.contains(child_id))
            .enumerate()
        {
            sqlx::query!(
                r#"
                    UPDATE term_taxonomy
                    SET weight = ?
                    WHERE parent_id = ?
                        AND id = ?
                "#,
                index as i32,
                payload.taxonomy_term_id,
                child_id,
            )
            .execute(&mut transaction)
            .await?;
        }

        let root = sqlx::query!(
            r#"
                SELECT tt.id, instance_id
                    FROM term_taxonomy tt
                    JOIN taxonomy t
                        ON t.id = tt.taxonomy_id
                    WHERE instance_id = (
                        SELECT instance_id
                        FROM term_taxonomy tt
                        JOIN taxonomy t
                            ON t.id = tt.taxonomy_id
                            WHERE tt.id = ?
                    )
                    AND t.type_id = 17
            "#,
            payload.taxonomy_term_id,
        )
        .fetch_one(&mut transaction)
        .await?;

        SetTaxonomyTermEventPayload::new(root.id as i32, payload.user_id, root.instance_id)
            .save(&mut transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }
}

impl AssertExists for TaxonomyTerm {}
