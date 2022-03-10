use crate::uuid::Subject;
use async_trait::async_trait;
use futures::try_join;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::HashMap;
use thiserror::Error;

use abstract_entity::AbstractEntity;
pub use entity_type::EntityType;

use super::entity_revision::abstract_entity_revision::EntityRevisionPayload;
use super::taxonomy_term::TaxonomyTerm;
use super::{ConcreteUuid, EntityRevision, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
use crate::event::{
    CreateEntityEventPayload, CreateEntityRevisionEventPayload, EntityLinkEventPayload, EventError,
    RevisionEventPayload,
};
use crate::format_alias;

use crate::instance::Instance;
use crate::subscription::Subscription;
use crate::uuid::abstract_entity_revision::EntityRevisionType;
pub use messages::*;

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityAddRevisionInput {
    pub changes: String,
    pub entity_id: i32,
    pub needs_review: bool,
    pub subscribe_this: bool,
    pub subscribe_this_by_email: bool,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityAddRevisionPayload {
    pub input: EntityAddRevisionInput,
    pub revision_type: EntityRevisionType,
    pub user_id: i32,
}

#[derive(Error, Debug)]
pub enum EntityAddRevisionError {
    #[error("Revision could not be added because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Revision could not be added because of an event error: {inner:?}.")]
    EventError { inner: EventError },
    #[error("Revision could not be added because of an UUID error: {inner:?}.")]
    UuidError { inner: UuidError },
    #[error("Revision could not be added because entity was not found.")]
    EntityNotFound,
}
impl From<sqlx::Error> for EntityAddRevisionError {
    fn from(inner: sqlx::Error) -> Self {
        Self::DatabaseError { inner }
    }
}

impl From<UuidError> for EntityAddRevisionError {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::DatabaseError { inner } => inner.into(),
            inner => Self::UuidError { inner },
        }
    }
}

impl From<EventError> for EntityAddRevisionError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => Self::EventError { inner },
        }
    }
}

impl Entity {
    pub async fn add_revision<'a, E>(
        payload: EntityAddRevisionPayload,
        executor: E,
    ) -> Result<Uuid, EntityAddRevisionError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        if let Err(UuidError::NotFound) =
            Entity::fetch_via_transaction(payload.input.entity_id, &mut transaction).await
        {
            return Err(EntityAddRevisionError::EntityNotFound);
        }

        let entity_revision = EntityRevisionPayload::new(
            payload.user_id,
            payload.input.entity_id,
            payload.input.changes,
            payload.input.fields,
        )
        .save(&mut transaction)
        .await?;
        if !payload.input.needs_review {
            let _ = Entity::checkout_revision(
                EntityCheckoutRevisionPayload {
                    revision_id: entity_revision.id,
                    user_id: payload.user_id,
                    reason: "".to_string(),
                },
                &mut transaction,
            )
            .await;
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
            payload.user_id,
            instance_id,
        )
        .save(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(entity_revision)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCreateInput {
    pub changes: String,
    pub instance: Instance,
    pub license_id: i32,
    pub subscribe_this: bool,
    pub needs_review: bool,
    pub subscribe_this_by_email: bool,
    pub fields: HashMap<String, String>,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCreatePayload {
    pub input: EntityCreateInput,
    pub entity_type: EntityType,
    pub user_id: i32,
}

#[derive(Error, Debug)]
pub enum EntityCreateError {
    #[error("Entity could not be created because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Entity could not be created because of an event error: {inner:?}.")]
    EventError { inner: EventError },
    #[error("Entity could not be created because of an event error: {inner:?}.")]
    AddRevisionError { inner: EntityAddRevisionError },
    #[error("Entity could not be created out because of an UUID error: {inner:?}.")]
    UuidError { inner: UuidError },
    #[error("Entity could not be created because parentId has to be provided.")]
    ParentIdError,
}

impl From<sqlx::Error> for EntityCreateError {
    fn from(inner: sqlx::Error) -> Self {
        Self::DatabaseError { inner }
    }
}

impl From<UuidError> for EntityCreateError {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::DatabaseError { inner } => inner.into(),
            inner => Self::UuidError { inner },
        }
    }
}

impl From<EventError> for EntityCreateError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => Self::EventError { inner },
        }
    }
}

impl From<EntityAddRevisionError> for EntityCreateError {
    fn from(error: EntityAddRevisionError) -> Self {
        match error {
            EntityAddRevisionError::DatabaseError { inner } => inner.into(),
            inner => Self::AddRevisionError { inner },
        }
    }
}

impl Entity {
    pub async fn create<'a, E>(
        payload: EntityCreatePayload,
        executor: E,
    ) -> Result<Uuid, EntityCreateError>
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

        let instance_id = Instance::fetch_id(&payload.input.instance, &mut transaction).await?;

        sqlx::query!(
            r#"
                INSERT INTO entity (id, type_id, instance_id, license_id)
                    VALUES (?, ?, ?, ?)
            "#,
            entity_id,
            type_id,
            instance_id,
            payload.input.license_id
        )
        .execute(&mut transaction)
        .await?;

        if let EntityType::CoursePage | EntityType::GroupedExercise | EntityType::Solution =
            payload.entity_type
        {
            let parent_id = payload
                .input
                .parent_id
                .ok_or(EntityCreateError::ParentIdError)?;

            sqlx::query!(
                r#"
                    INSERT INTO entity_link (parent_id, child_id, type_id)
                    VALUES (?, ?, 9)
                "#,
                parent_id,
                entity_id
            )
            .execute(&mut transaction)
            .await?;

            EntityLinkEventPayload::new(
                entity_id,
                parent_id,
                payload.user_id,
                payload.input.instance.clone(),
            )
            .save(&mut transaction)
            .await?;
        }

        let entity_revision = Entity::add_revision(
            EntityAddRevisionPayload {
                input: EntityAddRevisionInput {
                    changes: payload.input.changes,
                    entity_id,
                    needs_review: false,
                    subscribe_this: payload.input.subscribe_this,
                    subscribe_this_by_email: payload.input.subscribe_this_by_email,
                    fields: payload.input.fields.clone(),
                },
                revision_type: EntityRevisionType::from(payload.entity_type),
                user_id: payload.user_id,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        sqlx::query!(
            r#"
                UPDATE entity
                    SET current_revision_id = ?
                    WHERE id = ?
            "#,
            entity_revision.id,
            entity_id
        )
        .execute(&mut transaction)
        .await?;

        CreateEntityEventPayload::new(entity_id, payload.user_id, instance_id)
            .save(&mut transaction)
            .await?;

        let entity = Entity::fetch_via_transaction(entity_id, &mut transaction).await?;

        transaction.commit().await?;

        Ok(entity)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCheckoutRevisionPayload {
    pub revision_id: i32,
    pub user_id: i32,
    pub reason: String,
}

#[derive(Error, Debug)]
pub enum EntityCheckoutRevisionError {
    #[error("Revision could not be checked out because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Revision could not be checked out because of an event error: {inner:?}.")]
    EventError { inner: EventError },
    #[error("Revision could not be checked out because of an UUID error: {inner:?}.")]
    UuidError { inner: UuidError },
    #[error("Revision could not be checked out because it is already the current revision of its repository.")]
    RevisionAlreadyCheckedOut,
    #[error("Revision checkout failed because the provided UUID is not a revision: {uuid:?}.")]
    InvalidRevision { uuid: Uuid },
    #[error("Revision checkout failed because its repository is invalid: {uuid:?}.")]
    InvalidRepository { uuid: Uuid },
}

impl From<sqlx::Error> for EntityCheckoutRevisionError {
    fn from(inner: sqlx::Error) -> Self {
        Self::DatabaseError { inner }
    }
}

impl From<UuidError> for EntityCheckoutRevisionError {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::DatabaseError { inner } => inner.into(),
            inner => Self::UuidError { inner },
        }
    }
}

impl From<EventError> for EntityCheckoutRevisionError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => Self::EventError { inner },
        }
    }
}

impl Entity {
    pub async fn checkout_revision<'a, E>(
        payload: EntityCheckoutRevisionPayload,
        executor: E,
    ) -> Result<(), EntityCheckoutRevisionError>
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
                    return Err(EntityCheckoutRevisionError::RevisionAlreadyCheckedOut);
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
                    payload.reason,
                    abstract_entity.instance,
                )
                .save(&mut transaction)
                .await?;

                transaction.commit().await?;

                Ok(())
            } else {
                Err(EntityCheckoutRevisionError::InvalidRepository { uuid: repository })
            }
        } else {
            Err(EntityCheckoutRevisionError::InvalidRevision { uuid: revision })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRejectRevisionPayload {
    pub revision_id: i32,
    pub user_id: i32,
    pub reason: String,
}

#[derive(Error, Debug)]
pub enum EntityRejectRevisionError {
    #[error("Revision could not be rejected because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Revision could not be rejected because of an event error: {inner:?}.")]
    EventError { inner: EventError },
    #[error("Revision could not be rejected because of an UUID error: {inner:?}.")]
    UuidError { inner: UuidError },
    #[error("Revision could not be rejected out because it already has been rejected.")]
    RevisionAlreadyRejected,
    #[error("Revision could not be rejected out because it is checked out currently.")]
    RevisionCurrentlyCheckedOut,
    #[error(
        "Revision could not be rejected because the provided UUID is not a revision: {uuid:?}."
    )]
    InvalidRevision { uuid: Uuid },
    #[error("Revision could not be rejected because its repository is invalid: {uuid:?}.")]
    InvalidRepository { uuid: Uuid },
}

impl From<sqlx::Error> for EntityRejectRevisionError {
    fn from(inner: sqlx::Error) -> Self {
        Self::DatabaseError { inner }
    }
}

impl From<UuidError> for EntityRejectRevisionError {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::DatabaseError { inner } => inner.into(),
            inner => Self::UuidError { inner },
        }
    }
}

impl From<EventError> for EntityRejectRevisionError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::DatabaseError { inner } => inner.into(),
            inner => Self::EventError { inner },
        }
    }
}

impl Entity {
    pub async fn reject_revision<'a, E>(
        payload: EntityRejectRevisionPayload,
        executor: E,
    ) -> Result<(), EntityRejectRevisionError>
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
                return Err(EntityRejectRevisionError::RevisionAlreadyRejected);
            }

            let repository_id = abstract_entity_revision.repository_id;

            let repository = Entity::fetch_via_transaction(repository_id, &mut transaction).await?;

            if let ConcreteUuid::Entity(Entity {
                abstract_entity, ..
            }) = repository.concrete_uuid
            {
                if abstract_entity.current_revision_id == Some(revision_id) {
                    return Err(EntityRejectRevisionError::RevisionCurrentlyCheckedOut);
                }

                Uuid::set_state(revision_id, true, &mut transaction).await?;

                RevisionEventPayload::new(
                    true,
                    payload.user_id,
                    abstract_entity_revision.repository_id,
                    payload.revision_id,
                    payload.reason,
                    abstract_entity.instance,
                )
                .save(&mut transaction)
                .await?;

                transaction.commit().await?;

                Ok(())
            } else {
                Err(EntityRejectRevisionError::InvalidRepository { uuid: repository })
            }
        } else {
            Err(EntityRejectRevisionError::InvalidRevision { uuid: revision })
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnrevisedEntitiesQueryResult {
    pub unrevised_entity_ids: Vec<i32>,
}

impl Entity {
    pub async fn unrevised_entities<'a, E>(
        executor: E,
    ) -> Result<UnrevisedEntitiesQueryResult, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let unrevised_entity_ids = sqlx::query!(
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
        .collect();

        Ok(UnrevisedEntitiesQueryResult {
            unrevised_entity_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use std::collections::HashMap;

    use super::{
        Entity, EntityAddRevisionPayload, EntityCheckoutRevisionError,
        EntityCheckoutRevisionPayload, EntityRejectRevisionError, EntityRejectRevisionPayload,
        EntityRevision,
    };
    use crate::create_database_pool;
    use crate::event::test_helpers::fetch_age_of_newest_event;
    use crate::instance::Instance;
    use crate::subscription::tests::fetch_subscription_by_user_and_object;
    use crate::subscription::Subscription;
    use crate::uuid::abstract_entity_revision::EntityRevisionType;
    use crate::uuid::{
        ConcreteUuid, EntityAddRevisionInput, EntityCreateInput, EntityCreatePayload, EntityType,
        Uuid, UuidFetcher,
    };

    #[actix_rt::test]
    async fn add_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Entity::add_revision(
            EntityAddRevisionPayload {
                input: EntityAddRevisionInput {
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

        let revision_id = sqlx::query!(r#"SELECT id FROM entity_revision ORDER BY id desc limit 1"#)
            .fetch_one(&mut transaction)
            .await
            .unwrap()
            .id as i32;

        let revision = EntityRevision::fetch_via_transaction(revision_id, &mut transaction)
            .await
            .unwrap();

        if let ConcreteUuid::EntityRevision(EntityRevision {
            abstract_entity_revision,
            ..
        }) = revision.concrete_uuid
        {
            assert_eq!(abstract_entity_revision.changes, "test changes".to_string());
            assert_eq!(
                abstract_entity_revision.fields.0["title"],
                "test title".to_string()
            );
            assert_eq!(
                abstract_entity_revision.fields.0["content"],
                "test content".to_string()
            );
            assert_eq!(
                abstract_entity_revision.fields.0["meta_description"],
                "test meta-description".to_string()
            );
            assert_eq!(
                abstract_entity_revision.fields.0["meta_title"],
                "test meta-title".to_string()
            );
        } else {
            panic!(
                "Entity Revision does not fulfill assertions: {:?}",
                revision
            )
        }
    }

    #[actix_rt::test]
    async fn add_revision_needs_review_param() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Entity::add_revision(
            EntityAddRevisionPayload {
                input: EntityAddRevisionInput {
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

        let not_checked_out_revision_id =
            sqlx::query!(r#"SELECT id FROM entity_revision ORDER BY id desc limit 1"#)
                .fetch_one(&mut transaction)
                .await
                .unwrap()
                .id as i32;

        let entity = Entity::fetch_via_transaction(1495, &mut transaction)
            .await
            .unwrap();
        if let ConcreteUuid::Entity(Entity {
            abstract_entity, ..
        }) = entity.concrete_uuid
        {
            assert_ne!(
                abstract_entity.current_revision_id,
                Some(not_checked_out_revision_id)
            );
        } else {
            panic!("Entity does not fulfill assertions: {:?}", entity)
        }

        Entity::add_revision(
            EntityAddRevisionPayload {
                input: EntityAddRevisionInput {
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

        let checked_out_revision_id =
            sqlx::query!(r#"SELECT id FROM entity_revision ORDER BY id desc limit 1"#)
                .fetch_one(&mut transaction)
                .await
                .unwrap()
                .id as i32;

        let entity = Entity::fetch_via_transaction(1495, &mut transaction)
            .await
            .unwrap();
        if let ConcreteUuid::Entity(Entity {
            abstract_entity, ..
        }) = entity.concrete_uuid
        {
            assert_eq!(
                abstract_entity.current_revision_id,
                Some(checked_out_revision_id)
            );
        } else {
            panic!("Entity does not fulfill assertions: {:?}", entity)
        }
    }

    #[actix_rt::test]
    async fn add_revision_subscribe() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Entity::add_revision(
            EntityAddRevisionPayload {
                input: EntityAddRevisionInput {
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
    async fn create() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        let entity = Entity::create(
            EntityCreatePayload {
                input: EntityCreateInput {
                    changes: "test changes".to_string(),
                    instance: Instance::De,
                    subscribe_this: true,
                    needs_review: true,
                    subscribe_this_by_email: true,
                    license_id: 1,
                    fields: HashMap::from([
                        ("content".to_string(), "test content".to_string()),
                        (
                            "meta_description".to_string(),
                            "test meta-description".to_string(),
                        ),
                        ("meta_title".to_string(), "test meta-title".to_string()),
                        ("title".to_string(), "test title".to_string()),
                    ]),
                    parent_id: None,
                },
                user_id: 1,
                entity_type: EntityType::Article,
            },
            &mut transaction,
        )
        .await
        .unwrap();

        if let ConcreteUuid::Entity(Entity {
            abstract_entity, ..
        }) = entity.concrete_uuid
        {
            assert_eq!(abstract_entity.__typename, EntityType::Article);
            assert_eq!(abstract_entity.instance, Instance::De);
            assert_eq!(abstract_entity.license_id, 1);
        } else {
            panic!("Entity does not fulfill assertions: {:?}", entity)
        }
    }

    #[actix_rt::test]
    async fn checkout_revision() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        Entity::checkout_revision(
            EntityCheckoutRevisionPayload {
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
            EntityCheckoutRevisionPayload {
                revision_id: 30674,
                user_id: 1,
                reason: "Revert changes".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(EntityCheckoutRevisionError::RevisionAlreadyCheckedOut) = result {
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
            EntityRejectRevisionPayload {
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
            EntityRejectRevisionPayload {
                revision_id: 30672,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(EntityRejectRevisionError::RevisionAlreadyRejected) = result {
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
            EntityRejectRevisionPayload {
                revision_id: 30674,
                user_id: 1,
                reason: "Contains an error".to_string(),
            },
            &mut transaction,
        )
        .await;

        if let Err(EntityRejectRevisionError::RevisionCurrentlyCheckedOut) = result {
            // This is the expected branch.
        } else {
            panic!(
                "Expected `RevisionCurrentlyCheckedOut` error, got: {:?}",
                result
            )
        }
    }
}
