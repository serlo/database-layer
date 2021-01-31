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
use self::generic_entity_revision::GenericRevision;
use self::video_revision::VideoRevision;
use super::entity::Entity;
use super::{ConcreteUuid, Uuid, UuidError};

mod abstract_entity_revision;
mod applet_revision;
mod article_revision;
mod course_page_revision;
mod course_revision;
mod event_revision;
mod generic_entity_revision;
mod video_revision;

#[derive(Serialize)]
pub struct EntityRevision {
    #[serde(flatten)]
    pub abstract_entity_revision: AbstractEntityRevision,
    #[serde(flatten)]
    pub concrete_entity_revision: ConcreteEntityRevision,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ConcreteEntityRevision {
    Generic(GenericRevision),
    Applet(AppletRevision),
    Article(ArticleRevision),
    Course(CourseRevision),
    CoursePage(CoursePageRevision),
    Event(EventRevision),
    Video(VideoRevision),
}

impl EntityRevision {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let revision_fut = sqlx::query!(
            r#"
                SELECT t.name, u.trashed, r.date, r.author_id, r.repository_id
                    FROM entity_revision r
                    JOIN uuid u ON u.id = r.id
                    JOIN entity e ON e.id = r.repository_id
                    JOIN type t ON t.id = e.type_id
                    WHERE r.id = ?
            "#,
            id
        )
        .fetch_one(pool);
        let fields_fut = sqlx::query!(
            r#"
                SELECT field, value
                    FROM entity_revision_field
                    WHERE entity_revision_id = ?
            "#,
            id
        )
        .fetch_all(pool);
        let (revision, fields) = try_join!(revision_fut, fields_fut)?;

        let fields = fields
            .into_iter()
            .map(|field| (field.field, field.value))
            .collect();

        let fields = EntityRevisionFields(fields);

        let abstract_entity_revision = AbstractEntityRevision {
            __typename: revision.name.parse()?,
            date: revision.date.into(),
            author_id: revision.author_id as i32,
            repository_id: revision.repository_id as i32,
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
                ConcreteEntityRevision::Generic(abstract_entity_revision_ref.into())
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
            id,
            trashed: revision.trashed != 0,
            alias: format!(
                "/entity/repository/compare/{}/{}",
                revision.repository_id, id
            ),
            concrete_uuid: ConcreteUuid::EntityRevision(EntityRevision {
                abstract_entity_revision,
                concrete_entity_revision,
            }),
        })
    }

    pub async fn fetch_canonical_subject(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<String>, sqlx::Error> {
        let revision = sqlx::query!(
            r#"SELECT repository_id FROM entity_revision WHERE id = ?"#,
            id
        )
        .fetch_one(pool)
        .await?;
        Entity::fetch_canonical_subject(revision.repository_id as i32, pool).await
    }
}
