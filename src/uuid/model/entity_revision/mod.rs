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
use super::UuidError;
use crate::format_alias;

mod abstract_entity_revision;
mod applet_revision;
mod article_revision;
mod course_page_revision;
mod course_revision;
mod event_revision;
mod generic_entity_revision;
mod video_revision;

#[derive(Serialize)]
#[serde(untagged)]
pub enum EntityRevision {
    Generic(GenericRevision),
    Applet(AppletRevision),
    Article(ArticleRevision),
    Course(CourseRevision),
    CoursePage(CoursePageRevision),
    Event(EventRevision),
    Video(VideoRevision),
}

impl EntityRevision {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<EntityRevision, UuidError> {
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
            id,
            trashed: revision.trashed != 0,
            alias: format_alias(
                Self::fetch_canonical_subject(id, pool).await?.as_deref(),
                id,
                Some(&fields.get_or("title", &format!("{}", id))),
            ),
            date: revision.date.into(),
            author_id: revision.author_id as i32,
            repository_id: revision.repository_id as i32,
            changes: fields.get_or("changes", ""),
            fields,
        };

        let entity_revision = match abstract_entity_revision.__typename {
            EntityRevisionType::Applet => EntityRevision::Applet(abstract_entity_revision.into()),
            EntityRevisionType::Article => EntityRevision::Article(abstract_entity_revision.into()),
            EntityRevisionType::Course => EntityRevision::Course(abstract_entity_revision.into()),
            EntityRevisionType::CoursePage => {
                EntityRevision::CoursePage(abstract_entity_revision.into())
            }
            EntityRevisionType::Event => EntityRevision::Event(abstract_entity_revision.into()),
            EntityRevisionType::Exercise => {
                EntityRevision::Generic(abstract_entity_revision.into())
            }
            EntityRevisionType::ExerciseGroup => {
                EntityRevision::Generic(abstract_entity_revision.into())
            }
            EntityRevisionType::GroupedExercise => {
                EntityRevision::Generic(abstract_entity_revision.into())
            }
            EntityRevisionType::Solution => {
                EntityRevision::Generic(abstract_entity_revision.into())
            }
            EntityRevisionType::Video => EntityRevision::Video(abstract_entity_revision.into()),
        };

        Ok(entity_revision)
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

    pub fn get_alias(&self) -> String {
        match self {
            EntityRevision::Generic(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
            EntityRevision::Applet(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
            EntityRevision::Article(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
            EntityRevision::Course(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
            EntityRevision::CoursePage(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
            EntityRevision::Event(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
            EntityRevision::Video(entity_revision) => {
                entity_revision.abstract_entity_revision.alias.to_string()
            }
        }
    }
}
