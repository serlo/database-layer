use crate::database::Executor;
use async_trait::async_trait;
use convert_case::{Case, Casing};
use futures::join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::event::{
    CreateTaxonomyTermEventPayload, SetTaxonomyParentEventPayload, SetTaxonomyTermEventPayload,
};
use crate::instance::Instance;
use crate::uuid::model::taxonomy_term::messages::taxonomy_term_set_name_and_description_mutation;
use crate::{format_alias, operation};
pub use messages::*;

mod messages;

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
        .await?;

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

    pub async fn move_to_new_parent<'a, E>(
        payload: &taxonomy_term_move_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"SELECT * FROM term_taxonomy WHERE term_taxonomy.id = ?"#,
            payload.destination
        )
        .fetch_optional(&mut transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: format!(
                "Taxonomy term with id {} does not exist",
                payload.destination
            ),
        })?;

        for child_id in &payload.children_ids {
            if *child_id == payload.destination {
                return Err(operation::Error::BadRequest {
                    reason: format!(
                        "Child cannot have same id {} as destination",
                        payload.destination
                    ),
                });
            };

            let child = sqlx::query!(
                r#"
                    SELECT instance_id, parent_id AS previous_parent_id
                    FROM term_taxonomy
                    JOIN term
                    ON term.id = term_taxonomy.term_id
                    WHERE term_taxonomy.id = ?
                "#,
                child_id
            )
            .fetch_optional(&mut transaction)
            .await?
            .ok_or(operation::Error::BadRequest {
                reason: format!("Taxonomy term with id {} does not exist", child_id),
            })?;

            sqlx::query!(
                r#"
                    UPDATE term_taxonomy
                    SET parent_id = ?
                    WHERE id = ?
                "#,
                payload.destination,
                child_id,
            )
            .execute(&mut transaction)
            .await?;

            SetTaxonomyParentEventPayload::new(
                *child_id,
                child.previous_parent_id,
                payload.destination,
                payload.user_id,
                child.instance_id,
            )
            .save(&mut transaction)
            .await?;
        }

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

        let type_id = sqlx::query!(
            r#"SELECT id FROM type WHERE name = ?"#,
            payload.taxonomy_type
        )
        .fetch_one(&mut transaction)
        .await?
        .id as i32;

        let instance_id = Instance::fetch_id(&payload.instance, &mut transaction).await?;

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
        .await?;

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
            // TODO: missing weight
        )
        .execute(&mut transaction)
        .await?;

        CreateTaxonomyTermEventPayload::new(taxonomy_term_id, payload.user_id, instance_id)
            .save(&mut transaction)
            .await?;
        // should we use other methods to get the events as bonus?

        let taxonomy_term = Self::fetch_via_transaction(taxonomy_term_id, &mut transaction).await?;

        transaction.commit().await?;

        Ok(taxonomy_term)
    }
}
