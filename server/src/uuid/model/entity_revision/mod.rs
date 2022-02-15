use crate::uuid::Subject;
use async_trait::async_trait;
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use self::abstract_entity_revision::EntityRevisionFields;
use self::abstract_entity_revision::{AbstractEntityRevision, EntityRevisionType};
use self::applet_revision::AppletRevision;
use self::article_revision::ArticleRevision;
use self::course_page_revision::CoursePageRevision;
use self::course_revision::CourseRevision;
use self::event_revision::EventRevision;
use self::exercise_group_revision::ExerciseGroupRevision;
use self::generic_entity_revision::GenericRevision;
use self::video_revision::VideoRevision;
use super::entity::Entity;
use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;

pub mod abstract_entity_revision;
mod applet_revision;
mod article_revision;
mod course_page_revision;
mod course_revision;
mod event_revision;
mod exercise_group_revision;
mod generic_entity_revision;
mod video_revision;

#[derive(Debug, Serialize)]
pub struct EntityRevision {
    #[serde(flatten)]
    pub abstract_entity_revision: AbstractEntityRevision,
    #[serde(flatten)]
    pub concrete_entity_revision: ConcreteEntityRevision,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ConcreteEntityRevision {
    Generic(GenericRevision),
    Applet(AppletRevision),
    Article(ArticleRevision),
    Course(CourseRevision),
    CoursePage(CoursePageRevision),
    ExerciseGroupRevision(ExerciseGroupRevision),
    Event(EventRevision),
    Video(VideoRevision),
}

macro_rules! fetch_one_revision {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT t.name, u.trashed, r.date, r.author_id, r.repository_id
                    FROM entity_revision r
                    JOIN uuid u ON u.id = r.id
                    JOIN entity e ON e.id = r.repository_id
                    JOIN type t ON t.id = e.type_id
                    WHERE r.id = ?
            "#,
            $id
        )
        .fetch_one($executor)
    };
}

macro_rules! fetch_all_fields {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT field, value
                    FROM entity_revision_field
                    WHERE entity_revision_id = ?
            "#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! to_entity_revisions {
    ($id: expr, $revision: expr, $fields: expr) => {{
        let fields = $fields
            .into_iter()
            .map(|field| (field.field, field.value))
            .collect();

        let fields = EntityRevisionFields(fields);

        let abstract_entity_revision = AbstractEntityRevision {
            __typename: $revision.name.parse()?,
            date: $revision.date.into(),
            author_id: $revision.author_id as i32,
            repository_id: $revision.repository_id as i32,
            changes: fields.get_or("changes", ""),
            fields,
        };

        let abstract_entity_revision_ref = &abstract_entity_revision;

        let concrete_entity_revision = match abstract_entity_revision_ref.__typename {
            EntityRevisionType::Applet => {
                ConcreteEntityRevision::Applet(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::Article => {
                ConcreteEntityRevision::Article(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::Course => {
                ConcreteEntityRevision::Course(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::CoursePage => {
                ConcreteEntityRevision::CoursePage(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::Event => {
                ConcreteEntityRevision::Event(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::Exercise => {
                ConcreteEntityRevision::Generic(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::ExerciseGroup => {
                ConcreteEntityRevision::ExerciseGroupRevision(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::GroupedExercise => {
                ConcreteEntityRevision::Generic(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::Solution => {
                ConcreteEntityRevision::Generic(abstract_entity_revision_ref.into())
            }
            EntityRevisionType::Video => {
                ConcreteEntityRevision::Video(abstract_entity_revision_ref.into())
            }
        };

        Ok(Uuid {
            id: $id,
            trashed: $revision.trashed != 0,
            alias: format!(
                "/entity/repository/compare/{}/{}",
                $revision.repository_id, $id
            ),
            concrete_uuid: ConcreteUuid::EntityRevision(EntityRevision {
                abstract_entity_revision,
                concrete_entity_revision,
            }),
        })
    }};
}

#[async_trait]
impl UuidFetcher for EntityRevision {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let revision = fetch_one_revision!(id, pool);
        let fields = fetch_all_fields!(id, pool);
        let (revision, fields) = try_join!(revision, fields)?;
        to_entity_revisions!(id, revision, fields)
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let revision = fetch_one_revision!(id, &mut transaction).await?;
        let fields = fetch_all_fields!(id, &mut transaction).await?;
        transaction.commit().await?;
        to_entity_revisions!(id, revision, fields)
    }
}

impl EntityRevision {
    pub async fn fetch_canonical_subject(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<Subject>, sqlx::Error> {
        let revision = sqlx::query!(
            r#"SELECT repository_id FROM entity_revision WHERE id = ?"#,
            id
        )
        .fetch_one(pool)
        .await?;
        Entity::fetch_canonical_subject(revision.repository_id as i32, pool).await
    }

    pub async fn fetch_canonical_subject_via_transaction<'a, E>(
        id: i32,
        executor: E,
    ) -> Result<Option<Subject>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let revision = sqlx::query!(
            r#"SELECT repository_id FROM entity_revision WHERE id = ?"#,
            id
        )
        .fetch_one(&mut transaction)
        .await?;
        let subject = Entity::fetch_canonical_subject_via_transaction(
            revision.repository_id as i32,
            &mut transaction,
        )
        .await;
        transaction.commit().await?;
        subject
    }
}
