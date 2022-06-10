use crate::uuid::Subject;
use async_trait::async_trait;
use chrono_tz::Europe::Berlin;
use convert_case::{Case, Casing};
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::HashMap;

use abstract_entity::AbstractEntity;
pub use entity_type::EntityType;

use super::entity_revision::abstract_entity_revision::EntityRevisionPayload;
use super::taxonomy_term::TaxonomyTerm;
use super::{ConcreteUuid, EntityRevision, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::event::{
    CreateEntityEventPayload, CreateEntityRevisionEventPayload, CreateSetLicenseEventPayload,
    CreateTaxonomyLinkEventPayload, EntityLinkEventPayload, RevisionEventPayload,
};

use crate::{fetch_all_fields, format_alias};

use crate::datetime::DateTime;
use crate::operation;
use crate::subscription::Subscription;
use crate::uuid::abstract_entity_revision::EntityRevisionType;
pub use messages::*;

use crate::uuid::model::entity::messages::deleted_entities_query;

mod abstract_entity;
mod entity_type;
mod messages;

#[derive(Debug, Serialize)]
pub struct Entity {
    #[serde(flatten)]
    pub abstract_entity: AbstractEntity,
    #[serde(flatten)]
    pub concrete_entity: ConcreteEntity,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ConcreteEntity {
    Generic,
    Course(Course),
    CoursePage(CoursePage),
    ExerciseGroup(ExerciseGroup),
    Exercise(Exercise),
    GroupedExercise(GroupedExercise),
    Solution(Solution),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    page_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoursePage {
    parent_id: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseGroup {
    exercise_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    solution_id: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedExercise {
    parent_id: i32,
    solution_id: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
    parent_id: i32,
}

macro_rules! fetch_one_entity {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
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
            $id,
            $id
        )
        .fetch_one($executor)
    }
}

macro_rules! fetch_all_revisions {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"SELECT id FROM entity_revision WHERE repository_id = ?"#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! fetch_all_taxonomy_terms_parents {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"SELECT term_taxonomy_id as id FROM term_taxonomy_entity WHERE entity_id = ?"#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! to_entity {
    ($id: expr, $entity: expr, $revisions: expr, $taxonomy_terms: expr, $subject: expr, $executor: expr) => {{
        let abstract_entity = AbstractEntity {
            __typename: $entity.name.parse::<EntityType>()?,
            instance: $entity
                .subdomain
                .parse()
                .map_err(|_| UuidError::InvalidInstance)?,
            date: $entity.date.into(),
            license_id: $entity.license_id,
            taxonomy_term_ids: $taxonomy_terms.iter().map(|term| term.id as i32).collect(),
            canonical_subject_id: $subject
                .as_ref()
                .map(|subject| subject.taxonomy_term_id.clone()),

            current_revision_id: $entity.current_revision_id,
            revision_ids: $revisions
                .iter()
                .rev()
                .map(|revision| revision.id as i32)
                .collect(),
        };

        let concrete_entity = match abstract_entity.__typename {
            EntityType::Course => {
                let page_ids =
                    Entity::find_children_by_id_and_type($id, EntityType::CoursePage, $executor)
                        .await?;

                ConcreteEntity::Course(Course { page_ids })
            }
            EntityType::CoursePage => {
                let parent_id = Entity::find_parent_by_id($id, $executor).await?;

                ConcreteEntity::CoursePage(CoursePage { parent_id })
            }
            EntityType::ExerciseGroup => {
                let exercise_ids = Entity::find_children_by_id_and_type(
                    $id,
                    EntityType::GroupedExercise,
                    $executor,
                )
                .await?;

                ConcreteEntity::ExerciseGroup(ExerciseGroup { exercise_ids })
            }
            EntityType::Exercise => {
                let solution_id =
                    Entity::find_child_by_id_and_type($id, EntityType::Solution, $executor).await?;

                ConcreteEntity::Exercise(Exercise { solution_id })
            }
            EntityType::GroupedExercise => {
                let parent_id = Entity::find_parent_by_id($id, $executor).await?;
                let solution_id =
                    Entity::find_child_by_id_and_type($id, EntityType::Solution, $executor).await?;

                ConcreteEntity::GroupedExercise(GroupedExercise {
                    parent_id,
                    solution_id,
                })
            }
            EntityType::Solution => {
                let parent_id = Entity::find_parent_by_id_and_types(
                    $id,
                    [EntityType::Exercise, EntityType::GroupedExercise],
                    $executor,
                )
                .await?;

                ConcreteEntity::Solution(Solution { parent_id })
            }
            _ => ConcreteEntity::Generic,
        };

        Ok(Uuid {
            id: $id,
            trashed: $entity.trashed != 0,
            alias: format_alias(
                $subject.map(|subject| subject.name).as_deref(),
                $id,
                Some(
                    $entity
                        .title
                        .or($entity.fallback_title)
                        .unwrap_or(format!("{}", $id))
                        .as_str(),
                ),
            ),
            concrete_uuid: ConcreteUuid::Entity(Entity {
                abstract_entity,
                concrete_entity,
            }),
        })
    }};
}

macro_rules! fetch_all_taxonomy_terms_ancestors {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
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
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! fetch_canonical_subject {
    ($taxonomy_terms: expr, $executor: expr) => {
        match $taxonomy_terms.first() {
            Some(term) => TaxonomyTerm::fetch_canonical_subject(term.id as i32, $executor).await?,
            _ => None,
        }
    };
}

#[async_trait]
impl UuidFetcher for Entity {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let entity = fetch_one_entity!(id, pool);
        let revisions = fetch_all_revisions!(id, pool);
        let taxonomy_terms = fetch_all_taxonomy_terms_parents!(id, pool);
        let subject = Entity::fetch_canonical_subject(id, pool);
        let (entity, revisions, taxonomy_terms, subject) =
            try_join!(entity, revisions, taxonomy_terms, subject)?;

        to_entity!(id, entity, revisions, taxonomy_terms, subject, pool)
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let entity = fetch_one_entity!(id, &mut transaction).await?;
        let revisions = fetch_all_revisions!(id, &mut transaction).await?;
        let taxonomy_terms = fetch_all_taxonomy_terms_parents!(id, &mut transaction).await?;
        let subject = Entity::fetch_canonical_subject_via_transaction(id, &mut transaction).await?;
        let result = to_entity!(
            id,
            entity,
            revisions,
            taxonomy_terms,
            subject,
            &mut transaction
        );
        transaction.commit().await?;
        result
    }
}

impl Entity {
    pub async fn fetch_entity_type<'a, E>(
        id: i32,
        executor: E,
    ) -> Result<Option<EntityType>, UuidError>
    where
        E: Executor<'a>,
    {
        if let Some(result) = sqlx::query!(
            "select type.name from entity join type on type.id = entity.type_id where entity.id = ?",
            id
        )
        .fetch_optional(executor)
        .await? {
            Ok(Some(result.name.parse::<EntityType>()?))
        } else {
            Ok(None)
        }
    }

    pub async fn fetch_canonical_subject(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<Subject>, sqlx::Error> {
        let taxonomy_terms = fetch_all_taxonomy_terms_ancestors!(id, pool).await?;
        let subject = fetch_canonical_subject!(taxonomy_terms, pool);
        Ok(subject)
    }

    pub async fn fetch_canonical_subject_via_transaction<'a, E>(
        id: i32,
        executor: E,
    ) -> Result<Option<Subject>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let taxonomy_terms = fetch_all_taxonomy_terms_ancestors!(id, &mut transaction).await?;
        let subject = fetch_canonical_subject!(taxonomy_terms, &mut transaction);
        transaction.commit().await?;
        Ok(subject)
    }

    async fn find_parent_by_id<'a, E>(id: i32, executor: E) -> Result<i32, UuidError>
    where
        E: Executor<'a>,
    {
        let parents = sqlx::query!(
            r#"
                SELECT l.parent_id as id
                    FROM entity_link l
                    WHERE l.child_id = ?
            "#,
            id
        )
        .fetch_all(executor)
        .await?;
        parents
            .iter()
            .map(|parent| parent.id as i32)
            .collect::<Vec<_>>()
            .first()
            .ok_or(UuidError::EntityMissingRequiredParent)
            .map(|parent_id| *parent_id)
    }

    async fn find_parent_by_id_and_types<'a, E, const N: usize>(
        id: i32,
        parent_types: [EntityType; N],
        executor: E,
    ) -> Result<i32, UuidError>
    where
        E: Executor<'a>,
    {
        let query = format!(
            r#"
                SELECT l.parent_id as id
                    FROM entity_link l
                    JOIN entity e on l.parent_id = e.id
                    JOIN type t on t.id = e.type_id
                    WHERE l.child_id = ?
                        AND t.name in ({})
            "#,
            ["?"; N].join(", ")
        );

        let mut query = sqlx::query(&query);
        query = query.bind(id);
        for parent_type in parent_types.iter() {
            query = query.bind(parent_type);
        }

        let parent_id: i32 = query
            .fetch_one(executor)
            .await
            .and_then(|row| row.try_get(0))
            .map_err(|_| UuidError::EntityMissingRequiredParent)?;

        Ok(parent_id)
    }

    async fn find_children_by_id_and_type<'a, E>(
        id: i32,
        child_type: EntityType,
        executor: E,
    ) -> Result<Vec<i32>, UuidError>
    where
        E: Executor<'a>,
    {
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
            child_type,
        )
        .fetch_all(executor)
        .await?;
        Ok(children.iter().map(|child| child.id as i32).collect())
    }

    async fn find_child_by_id_and_type<'a, E>(
        id: i32,
        child_type: EntityType,
        executor: E,
    ) -> Result<Option<i32>, UuidError>
    where
        E: Executor<'a>,
    {
        Self::find_children_by_id_and_type(id, child_type, executor)
            .await
            .map(|children| children.first().cloned())
    }
}

impl Entity {
    pub async fn add_revision<'a, E>(
        payload: &entity_add_revision_mutation::Payload,
        executor: E,
    ) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        Self::assert_entity_exists(payload.input.entity_id, &mut transaction).await?;

        let last_not_trashed_revision = sqlx::query!(
            r#"
            SELECT er.id
                FROM entity_revision er
                JOIN uuid ON er.id = uuid.id
                WHERE repository_id = ?
                    AND trashed = 0
                ORDER BY date DESC
                LIMIT 1
            "#,
            payload.input.entity_id
        )
        .fetch_optional(&mut transaction)
        .await?
        .map(|x| x.id as i32);

        let mut fields = payload.input.fields.clone();

        if let Some(revision_id) = last_not_trashed_revision {
            let mut last_revision_fields: HashMap<String, String> =
                fetch_all_fields!(revision_id, &mut transaction)
                    .await?
                    .into_iter()
                    .filter(|field| field.field != "changes")
                    .map(|field| (field.field.to_case(Case::Camel), field.value))
                    .collect();

            // FIXME: This is bad design -> let's have a DB migration?!
            if payload.revision_type == EntityRevisionType::ExerciseGroup
                && !last_revision_fields.contains_key("cohesive")
                && fields.contains_key("cohesive")
            {
                last_revision_fields.insert("cohesive".to_string(), "false".to_string());
            }

            // FIXME: Do we need to fix this in the frontend?!
            if payload.revision_type == EntityRevisionType::CoursePage
                && !fields.contains_key("icon")
            {
                fields.insert("icon".to_string(), "book-open".to_string());
            }

            if last_revision_fields == fields {
                return Ok(
                    EntityRevision::fetch_via_transaction(revision_id, &mut transaction).await?,
                );
            }
        }

        fields.insert("changes".to_string(), payload.input.changes.clone());

        let entity_revision =
            EntityRevisionPayload::new(payload.user_id, payload.input.entity_id, fields)
                .save(&mut transaction)
                .await?;

        let instance_id = sqlx::query!(
            r#"
                SELECT instance_id
                    FROM entity
                    WHERE id = ?
            "#,
            payload.input.entity_id
        )
        .fetch_one(&mut transaction)
        .await?
        .instance_id as i32;

        CreateEntityRevisionEventPayload::new(
            payload.input.entity_id,
            entity_revision.id,
            payload.user_id,
            instance_id,
        )
        .save(&mut transaction)
        .await?;

        if !payload.input.needs_review {
            Entity::checkout_revision(
                &checkout_revision_mutation::Payload {
                    revision_id: entity_revision.id,
                    user_id: payload.user_id,
                    reason: "".to_string(),
                },
                &mut transaction,
            )
            .await
            .map_err(|error| operation::Error::InternalServerError {
                error: Box::new(error),
            })?;
        }

        if payload.input.subscribe_this {
            Subscription::save(
                &Subscription {
                    object_id: payload.input.entity_id,
                    user_id: payload.user_id,
                    send_email: payload.input.subscribe_this_by_email,
                },
                &mut transaction,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(entity_revision)
    }

    pub async fn assert_entity_exists<'a, E>(id: i32, executor: E) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        sqlx::query!(r#"SELECT id FROM entity WHERE id = ?"#, id)
            .fetch_one(executor)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => operation::Error::BadRequest {
                    reason: format!("Entity with id {} does not exist", id),
                },
                _ => operation::Error::InternalServerError {
                    error: Box::new(error),
                },
            })?;

        Ok(())
    }

    pub async fn create<'a, E>(
        payload: &entity_create_mutation::Payload,
        executor: E,
    ) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator)
                    VALUES (0, "entity")
            "#,
        )
        .execute(&mut transaction)
        .await?;

        let entity_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        let type_id = sqlx::query!(r#"SELECT id FROM type WHERE name = ?"#, payload.entity_type)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        let parent_id: i32;
        let instance_id: i32;

        if let EntityType::CoursePage | EntityType::GroupedExercise | EntityType::Solution =
            payload.entity_type
        {
            parent_id = payload
                .input
                .parent_id
                .ok_or(operation::Error::BadRequest {
                    reason: "parent_id needs to be provided".to_string(),
                })?;

            instance_id = sqlx::query!("select instance_id from entity where id = ?", parent_id)
                .fetch_optional(&mut transaction)
                .await?
                .ok_or(operation::Error::BadRequest {
                    reason: format!("parent entity with id {} does not exist", parent_id),
                })?
                .instance_id as i32;
        } else {
            parent_id = payload
                .input
                .taxonomy_term_id
                .ok_or(operation::Error::BadRequest {
                    reason: "taxonomy_term_id needs to be provided".to_string(),
                })?;

            instance_id = TaxonomyTerm::get_instance_id(parent_id, &mut transaction).await?;
        }

        sqlx::query!(
            r#"
                INSERT INTO entity (id, type_id, instance_id, license_id, date)
                    VALUES (?, ?, ?, ?, ?)
            "#,
            entity_id,
            type_id,
            instance_id,
            payload.input.license_id,
            DateTime::now(),
        )
        .execute(&mut transaction)
        .await?;

        if let EntityType::CoursePage | EntityType::GroupedExercise | EntityType::Solution =
            payload.entity_type
        {
            let last_order = sqlx::query!(
                r#"
                    SELECT IFNULL(MAX(et.order), 0) AS current_last
                        FROM entity_link et
                        WHERE et.parent_id = ?
                "#,
                parent_id,
            )
            .fetch_one(&mut transaction)
            .await?
            .current_last as i32
                + 1;

            sqlx::query!(
                r#"
                    INSERT INTO entity_link (parent_id, child_id, type_id, entity_link.order)
                    VALUES (?, ?, 9, ?)
                "#,
                parent_id,
                entity_id,
                last_order
            )
            .execute(&mut transaction)
            .await?;

            EntityLinkEventPayload::new(entity_id, parent_id, payload.user_id, instance_id)
                .save(&mut transaction)
                .await?;
        } else {
            let last_position = sqlx::query!(
                r#"
                    SELECT IFNULL(MAX(position), 0) AS current_last
                        FROM term_taxonomy_entity
                        WHERE term_taxonomy_id = ?
                "#,
                parent_id
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
                entity_id,
                parent_id,
                last_position
            )
            .execute(&mut transaction)
            .await?;

            CreateTaxonomyLinkEventPayload::new(entity_id, parent_id, payload.user_id, instance_id)
                .save(&mut transaction)
                .await?;
        }

        CreateEntityEventPayload::new(entity_id, payload.user_id, instance_id)
            .save(&mut transaction)
            .await?;

        Entity::add_revision(
            &entity_add_revision_mutation::Payload {
                input: entity_add_revision_mutation::Input {
                    changes: payload.input.changes.clone(),
                    entity_id,
                    needs_review: payload.input.needs_review,
                    subscribe_this: payload.input.subscribe_this,
                    subscribe_this_by_email: payload.input.subscribe_this_by_email,
                    fields: payload.input.fields.clone(),
                },
                revision_type: EntityRevisionType::from(payload.entity_type.clone()),
                user_id: payload.user_id,
            },
            &mut transaction,
        )
        .await?;

        let entity = Entity::fetch_via_transaction(entity_id, &mut transaction).await?;

        transaction.commit().await?;

        Ok(entity)
    }
}

impl Entity {
    pub async fn checkout_revision<'a, E>(
        payload: &checkout_revision_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let revision_id = payload.revision_id;
        let revision = EntityRevision::fetch_via_transaction(revision_id, &mut transaction).await?;

        if let ConcreteUuid::EntityRevision(EntityRevision {
            abstract_entity_revision,
            ..
        }) = revision.concrete_uuid
        {
            let repository_id = abstract_entity_revision.repository_id;

            let repository = Entity::fetch_via_transaction(repository_id, &mut transaction).await?;

            if let ConcreteUuid::Entity(Entity {
                abstract_entity, ..
            }) = repository.concrete_uuid
            {
                if abstract_entity.current_revision_id == Some(revision_id) {
                    return Err(operation::Error::BadRequest {
                        reason: "revision is already checked out".to_string(),
                    });
                }

                Uuid::set_state(revision_id, false, &mut transaction).await?;

                sqlx::query!(
                    r#"
                        UPDATE entity
                            SET current_revision_id = ?
                            WHERE id = ?
                    "#,
                    revision_id,
                    repository_id,
                )
                .execute(&mut transaction)
                .await?;

                RevisionEventPayload::new(
                    false,
                    payload.user_id,
                    repository_id,
                    payload.revision_id,
                    payload.reason.clone(),
                    abstract_entity.instance,
                )
                .save(&mut transaction)
                .await?;

                transaction.commit().await?;

                Ok(())
            } else {
                Err(operation::Error::BadRequest {
                    reason: "repository  invalid".to_string(),
                })
            }
        } else {
            Err(operation::Error::BadRequest {
                reason: "revision invalid".to_string(),
            })
        }
    }
}

impl Entity {
    pub async fn reject_revision<'a, E>(
        payload: &reject_revision_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let revision_id = payload.revision_id;
        let revision = EntityRevision::fetch_via_transaction(revision_id, &mut transaction).await?;

        if let ConcreteUuid::EntityRevision(EntityRevision {
            abstract_entity_revision,
            ..
        }) = revision.concrete_uuid
        {
            if revision.trashed {
                return Err(operation::Error::BadRequest {
                    reason: "revision is already rejected".to_string(),
                });
            }

            let repository_id = abstract_entity_revision.repository_id;

            let repository = Entity::fetch_via_transaction(repository_id, &mut transaction).await?;

            if let ConcreteUuid::Entity(Entity {
                abstract_entity, ..
            }) = repository.concrete_uuid
            {
                if abstract_entity.current_revision_id == Some(revision_id) {
                    return Err(operation::Error::BadRequest {
                        reason: "revision is checked out currently".to_string(),
                    });
                }

                Uuid::set_state(revision_id, true, &mut transaction).await?;

                RevisionEventPayload::new(
                    true,
                    payload.user_id,
                    abstract_entity_revision.repository_id,
                    payload.revision_id,
                    payload.reason.clone(),
                    abstract_entity.instance,
                )
                .save(&mut transaction)
                .await?;

                transaction.commit().await?;

                Ok(())
            } else {
                Err(operation::Error::BadRequest {
                    reason: "repository invalid".to_string(),
                })
            }
        } else {
            Err(operation::Error::BadRequest {
                reason: "revision invalid".to_string(),
            })
        }
    }
}

impl Entity {
    pub async fn unrevised_entities<'a, E>(executor: E) -> Result<Vec<i32>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        Ok(sqlx::query!(
            r#"
                SELECT
                    MIN(e.id) as entity_id,
                    MIN(r.id) as min_revision_id
                FROM entity_revision r
                JOIN uuid u_r ON r.id = u_r.id
                JOIN entity e ON e.id = r.repository_id
                JOIN uuid u_e ON e.id = u_e.id
                WHERE ( e.current_revision_id IS NULL OR r.id > e.current_revision_id )
                    AND u_r.trashed = 0
                    AND u_e.trashed = 0
                GROUP BY e.id
                ORDER BY min_revision_id
            "#,
        )
        .fetch_all(executor)
        .await?
        .iter()
        .map(|record| record.entity_id.unwrap() as i32)
        .collect())
    }
}

impl Entity {
    pub async fn deleted_entities<'a, E>(
        payload: &deleted_entities_query::Payload,
        executor: E,
    ) -> Result<Vec<deleted_entities_query::DeletedEntity>, operation::Error>
    where
        E: Executor<'a>,
    {
        let after_db_time = payload
            .after
            .as_ref()
            .map(|after| {
                chrono::DateTime::parse_from_rfc3339(after).map_err(|_| {
                    operation::Error::BadRequest {
                        reason: "The date format should be YYYY-MM-DDThh:mm:ss{Timezone}"
                            .to_string(),
                    }
                })
            })
            .transpose()?
            .map(|date| date.with_timezone(&Berlin).to_string());

        Ok(sqlx::query!(
            r#"
                SELECT uuid_id, MAX(event_log.date) AS date
                FROM event_log, uuid, instance, entity
                WHERE uuid.id = event_log.uuid_id
                    AND event_log.date < ?
                    AND (? is null OR instance.subdomain = ?)
                    AND instance.id = entity.instance_id
                    AND entity.id = event_log.uuid_id
                    AND event_log.event_id = 10
                    AND uuid.trashed = 1
                    AND uuid.discriminator = 'entity'
                    AND entity.type_id NOT IN (35, 39, 40, 41, 42, 43, 44)
                GROUP BY uuid_id
                ORDER BY date DESC
                LIMIT ?
            "#,
            after_db_time.unwrap_or(DateTime::now().to_string()),
            payload.instance,
            payload.instance,
            payload.first,
        )
        .fetch_all(executor)
        .await?
        .into_iter()
        .filter_map(|result| {
            result
                .date
                .map(|date| deleted_entities_query::DeletedEntity {
                    id: result.uuid_id as i32,
                    date_of_deletion: DateTime::from(date).to_string(),
                })
        })
        .collect())
    }
}

impl Entity {
    pub async fn sort<'a, E>(
        payload: &entity_sort_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        /*Self::assert_exists(payload.taxonomy_term_id, &mut transaction).await?;

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

        if !HashSet::from_iter(payload.children_ids.clone())
            .is_subset(&HashSet::<i32>::from_iter(children_ids.clone()))
        {
            return Err(operation::Error::BadRequest {
                reason: "children_ids have to be a subset of children entities and taxonomy terms of the given taxonomy term".to_string()
            });
        }*/

        for (index, child_id) in payload.children_ids.iter().enumerate() {
            sqlx::query!(
                r#"
                    UPDATE entity_link
                    SET entity_link.order = ?
                    WHERE parent_id = ? AND child_id = ?
                "#,
                index as i32,
                payload.entity_id,
                child_id,
            )
            .execute(&mut transaction)
            .await?;
        }

        /*
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
            .await?;*/

        transaction.commit().await?;

        Ok(())
    }
}

impl Entity {
    pub async fn set_license<'a, E>(
        payload: &entity_set_license_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                select * from user where id = ?
            "#,
            payload.user_id,
        )
        .fetch_optional(&mut transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: format!("An user with id {} does not exist.", payload.user_id),
        })?;

        let entity = sqlx::query!(
            r#"
                select * from entity where id = ?
            "#,
            payload.entity_id,
        )
        .fetch_optional(&mut transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: format!("An entity with id {} does not exist.", payload.entity_id),
        })?;

        if entity.license_id == payload.license_id {
            return Ok(());
        }

        sqlx::query!(
            r#"
                update entity set license_id = ? where id = ?
            "#,
            payload.license_id,
            payload.entity_id,
        )
        .execute(&mut transaction)
        .await?;

        CreateSetLicenseEventPayload::new(payload.entity_id, payload.user_id, entity.instance_id)
            .save(&mut transaction)
            .await?;

        transaction.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use std::collections::HashMap;

    use super::*;
    use crate::event::test_helpers::fetch_age_of_newest_event;
    use crate::subscription::tests::fetch_subscription_by_user_and_object;
    use crate::subscription::Subscription;
    use crate::uuid::abstract_entity_revision::EntityRevisionType;
    use crate::uuid::{entity_add_revision_mutation, ConcreteUuid, Uuid, UuidFetcher};
    use crate::{create_database_pool, operation};

    #[actix_rt::test]
    async fn check_entity_exists_throws_bad_request_error() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        match Entity::assert_entity_exists(1, &mut transaction).await {
            Err(error) => match error {
                operation::Error::BadRequest { reason: _ } => {}
                _ => panic!("check_entity_exists didn't throw expected error"),
            },
            _ => panic!("check_entity_exists didn't throw expected error"),
        }
    }

    #[actix_rt::test]
    async fn add_revision_when_needs_review_is_true() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let new_revision = Entity::add_revision(
            &entity_add_revision_mutation::Payload {
                input: entity_add_revision_mutation::Input {
                    changes: "test changes".to_string(),
                    entity_id: 1495,
                    needs_review: true,
                    subscribe_this: false,
                    subscribe_this_by_email: false,
                    fields: HashMap::from([
                        ("content".to_string(), "test content".to_string()),
                        (
                            "meta_description".to_string(),
                            "test meta-description".to_string(),
                        ),
                        ("meta_title".to_string(), "test meta-title".to_string()),
                        ("title".to_string(), "test title".to_string()),
                    ]),
                },
                revision_type: EntityRevisionType::Article,
                user_id: 1,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        let entity = Entity::fetch_via_transaction(1495, &mut transaction)
            .await
            .unwrap();

        if let ConcreteUuid::Entity(Entity {
            abstract_entity, ..
        }) = entity.concrete_uuid
        {
            assert_ne!(abstract_entity.current_revision_id, Some(new_revision.id));
        } else {
            panic!("Entity does not fulfill assertions: {:?}", entity)
        }
    }

    #[actix_rt::test]
    async fn add_revision_when_needs_review_is_false() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let new_revision = Entity::add_revision(
            &entity_add_revision_mutation::Payload {
                input: entity_add_revision_mutation::Input {
                    changes: "test changes".to_string(),
                    entity_id: 1495,
                    needs_review: false,
                    subscribe_this: false,
                    subscribe_this_by_email: false,
                    fields: HashMap::from([
                        ("content".to_string(), "test content".to_string()),
                        (
                            "meta_description".to_string(),
                            "test meta-description".to_string(),
                        ),
                        ("meta_title".to_string(), "test meta-title".to_string()),
                        ("title".to_string(), "test title".to_string()),
                    ]),
                },
                revision_type: EntityRevisionType::Article,
                user_id: 1,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        let entity = Entity::fetch_via_transaction(1495, &mut transaction)
            .await
            .unwrap();
        if let ConcreteUuid::Entity(Entity {
            abstract_entity, ..
        }) = entity.concrete_uuid
        {
            assert_eq!(abstract_entity.current_revision_id, Some(new_revision.id));
        } else {
            panic!("Entity does not fulfill assertions: {:?}", entity)
        }
    }

    #[actix_rt::test]
    async fn add_revision_subscribe() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Entity::add_revision(
            &entity_add_revision_mutation::Payload {
                input: entity_add_revision_mutation::Input {
                    changes: "test changes".to_string(),
                    entity_id: 1497,
                    needs_review: true,
                    subscribe_this: true,
                    subscribe_this_by_email: true,
                    fields: HashMap::from([
                        ("content".to_string(), "test content".to_string()),
                        (
                            "meta_description".to_string(),
                            "test meta-description".to_string(),
                        ),
                        ("meta_title".to_string(), "test meta-title".to_string()),
                        ("title".to_string(), "test title".to_string()),
                    ]),
                },
                revision_type: EntityRevisionType::Article,
                user_id: 1,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        let entity_subscription = fetch_subscription_by_user_and_object(1, 1497, &mut transaction)
            .await
            .unwrap();

        assert_eq!(
            entity_subscription,
            Some(Subscription {
                object_id: 1497,
                user_id: 1,
                send_email: true
            })
        );
    }

    #[actix_rt::test]
    async fn checkout_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Entity::checkout_revision(
            &checkout_revision_mutation::Payload {
                revision_id: 30672,
                user_id: 1,
                reason: "Revert changes".to_string(),
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that revision was checked out.
        let entity = Entity::fetch_via_transaction(1855, &mut transaction)
            .await
            .unwrap();
        if let ConcreteUuid::Entity(Entity {
            abstract_entity, ..
        }) = entity.concrete_uuid
        {
            assert_eq!(abstract_entity.current_revision_id, Some(30672));
        } else {
            panic!("Entity does not fulfill assertions: {:?}", entity)
        }

        // Verify that the event was created.
        let duration = fetch_age_of_newest_event(30672, &mut transaction)
            .await
            .unwrap();
        assert!(duration < Duration::minutes(1));
    }

    #[actix_rt::test]
    async fn checkout_revision_sets_trashed_flag_to_false() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let revision_id: i32 = 30672;
        let entity_id: i32 = 1855;

        Uuid::set_state(revision_id, true, &mut transaction)
            .await
            .unwrap();

        let entity = Entity::fetch_via_transaction(entity_id, &mut transaction)
            .await
            .unwrap();
        assert!(!entity.trashed);
    }

    #[actix_rt::test]
    async fn checkout_checked_out_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let result = Entity::checkout_revision(
            &checkout_revision_mutation::Payload {
                revision_id: 30674,
                user_id: 1,
                reason: "Revert changes".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(operation::Error::BadRequest { .. }) = result {
            // This is the expected branch.
        } else {
            panic!(
                "Expected `RevisionAlreadyCheckedOut` error, got: {:?}",
                result
            )
        }
    }

    #[actix_rt::test]
    async fn reject_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Entity::reject_revision(
            &reject_revision_mutation::Payload {
                revision_id: 30672,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await
        .unwrap();

        // Verify that revision was trashed.
        let revision = EntityRevision::fetch_via_transaction(30672, &mut transaction)
            .await
            .unwrap();
        assert!(revision.trashed);

        // Verify that the event was created.
        let duration = fetch_age_of_newest_event(30672, &mut transaction)
            .await
            .unwrap();
        assert!(duration < Duration::minutes(1));
    }

    #[actix_rt::test]
    async fn reject_rejected_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Uuid::set_state(30672, true, &mut transaction)
            .await
            .unwrap();

        let result = Entity::reject_revision(
            &reject_revision_mutation::Payload {
                revision_id: 30672,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(operation::Error::BadRequest { .. }) = result {
            // This is the expected branch.
        } else {
            panic!(
                "Expected `RevisionAlreadyRejected` error, got: {:?}",
                result
            )
        }
    }

    #[actix_rt::test]
    async fn reject_checked_out_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let result = Entity::reject_revision(
            &reject_revision_mutation::Payload {
                revision_id: 30674,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(operation::Error::BadRequest { .. }) = result {
            // This is the expected branch.
        } else {
            panic!(
                "Expected `RevisionCurrentlyCheckedOut` error, got: {:?}",
                result
            )
        }
    }
}
