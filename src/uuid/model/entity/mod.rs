use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use abstract_entity::{AbstractEntity, EntityType};

use super::taxonomy_term::TaxonomyTerm;
use super::UuidError;
use crate::format_alias;

mod abstract_entity;

#[derive(Serialize)]
pub struct Entity {
    #[serde(flatten)]
    pub abstract_entity: AbstractEntity,
    #[serde(flatten)]
    pub concrete_entity: ConcreteEntity,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    page_ids: Vec<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoursePage {
    parent_id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseGroup {
    exercise_ids: Vec<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    solution_id: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedExercise {
    parent_id: i32,
    solution_id: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
    parent_id: i32,
}

impl Entity {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Entity, UuidError> {
        let entity_fut = sqlx::query!(
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
            id,
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
        let subject_fut = Self::fetch_canonical_subject(id, pool);
        let (entity, revisions, taxonomy_terms, subject) =
            try_join!(entity_fut, revisions_fut, taxonomy_terms_fut, subject_fut)?;

        let abstract_entity = AbstractEntity {
            __typename: entity.name.parse::<EntityType>()?,
            id,
            trashed: entity.trashed != 0,
            alias: format_alias(
                subject.as_deref(),
                id,
                Some(
                    entity
                        .title
                        .or(entity.fallback_title)
                        .unwrap_or(format!("{}", id))
                        .as_str(),
                ),
            ),
            instance: entity
                .subdomain
                .parse()
                .map_err(|_| UuidError::InvalidInstance)?,
            date: entity.date.into(),
            license_id: entity.license_id,
            taxonomy_term_ids: taxonomy_terms.iter().map(|term| term.id as i32).collect(),

            current_revision_id: entity.current_revision_id,
            revision_ids: revisions
                .iter()
                .rev()
                .map(|revision| revision.id as i32)
                .collect(),
        };

        let concrete_entity = match abstract_entity.__typename {
            EntityType::Course => {
                let page_ids =
                    Self::find_children_by_id_and_type(id, EntityType::CoursePage, pool).await?;

                ConcreteEntity::Course(Course { page_ids })
            }
            EntityType::CoursePage => {
                let parent_id = Self::find_parent_by_id(id, pool).await?;

                ConcreteEntity::CoursePage(CoursePage { parent_id })
            }
            EntityType::ExerciseGroup => {
                let exercise_ids =
                    Self::find_children_by_id_and_type(id, EntityType::GroupedExercise, pool)
                        .await?;

                ConcreteEntity::ExerciseGroup(ExerciseGroup { exercise_ids })
            }
            EntityType::Exercise => {
                let solution_id =
                    Self::find_child_by_id_and_type(id, EntityType::Solution, pool).await?;

                ConcreteEntity::Exercise(Exercise { solution_id })
            }
            EntityType::GroupedExercise => {
                let parent_id = Self::find_parent_by_id(id, pool).await?;
                let solution_id =
                    Self::find_child_by_id_and_type(id, EntityType::Solution, pool).await?;

                ConcreteEntity::GroupedExercise(GroupedExercise {
                    parent_id,
                    solution_id,
                })
            }
            EntityType::Solution => {
                let parent_id = Self::find_parent_by_id(id, pool).await?;

                ConcreteEntity::Solution(Solution { parent_id })
            }
            _ => ConcreteEntity::Generic,
        };

        Ok(Entity {
            abstract_entity,
            concrete_entity,
        })
    }

    pub async fn fetch_canonical_subject(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<String>, sqlx::Error> {
        let taxonomy_terms = sqlx::query!(
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
            id
        )
        .fetch_all(pool)
        .await?;
        let subject = match taxonomy_terms.first() {
            Some(term) => TaxonomyTerm::fetch_canonical_subject(term.id as i32, pool).await?,
            _ => None,
        };
        Ok(subject)
    }

    async fn find_parent_by_id(id: i32, pool: &MySqlPool) -> Result<i32, UuidError> {
        let parents = sqlx::query!(
            r#"
                SELECT l.parent_id as id
                    FROM entity_link l
                    WHERE l.child_id = ?
            "#,
            id
        )
        .fetch_all(pool)
        .await?;
        parents
            .iter()
            .map(|parent| parent.id as i32)
            .collect::<Vec<i32>>()
            .first()
            .ok_or(UuidError::EntityMissingRequiredParent)
            .map(|parent_id| *parent_id)
    }

    async fn find_children_by_id_and_type(
        id: i32,
        child_type: EntityType,
        pool: &MySqlPool,
    ) -> Result<Vec<i32>, UuidError> {
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
        .fetch_all(pool)
        .await?;
        Ok(children.iter().map(|child| child.id as i32).collect())
    }

    async fn find_child_by_id_and_type(
        id: i32,
        child_type: EntityType,
        pool: &MySqlPool,
    ) -> Result<Option<i32>, UuidError> {
        Self::find_children_by_id_and_type(id, child_type, pool)
            .await
            .map(|children| children.first().cloned())
    }
}
