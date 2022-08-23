use convert_case::{Case, Casing};
use std::collections::HashMap;

use crate::database::Executor;
use serde::{Deserialize, Serialize};

use super::UuidError;
use crate::datetime::DateTime;
use crate::operation;
use crate::uuid::{EntityRevision, EntityType, Uuid, UuidFetcher};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEntityRevision {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EntityRevisionType,
    pub date: DateTime,
    pub author_id: i32,
    pub repository_id: i32,
    pub changes: String,

    #[serde(skip)]
    pub fields: EntityRevisionFields,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub enum EntityRevisionType {
    #[serde(rename = "AppletRevision")]
    Applet,
    #[serde(rename = "ArticleRevision")]
    Article,
    #[serde(rename = "CourseRevision")]
    Course,
    #[serde(rename = "CoursePageRevision")]
    CoursePage,
    #[serde(rename = "EventRevision")]
    Event,
    #[serde(rename = "ExerciseRevision")]
    Exercise,
    #[serde(rename = "ExerciseGroupRevision")]
    ExerciseGroup,
    #[serde(rename = "GroupedExerciseRevision")]
    GroupedExercise,
    #[serde(rename = "SolutionRevision")]
    Solution,
    #[serde(rename = "VideoRevision")]
    Video,
}

impl From<EntityType> for EntityRevisionType {
    fn from(entity_type: EntityType) -> Self {
        match entity_type {
            EntityType::Applet => Self::Applet,
            EntityType::Article => Self::Article,
            EntityType::Course => Self::Course,
            EntityType::CoursePage => Self::CoursePage,
            EntityType::Event => Self::Event,
            EntityType::Exercise => Self::Exercise,
            EntityType::ExerciseGroup => Self::ExerciseGroup,
            EntityType::GroupedExercise => Self::GroupedExercise,
            EntityType::Solution => Self::Solution,
            EntityType::Video => Self::Video,
        }
    }
}

impl std::str::FromStr for EntityRevisionType {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entity_type = EntityType::from_str(s)?;
        Ok(entity_type.into())
    }
}

#[derive(Debug)]
pub struct EntityRevisionFields(pub HashMap<String, String>);

impl EntityRevisionFields {
    pub fn get_or(&self, name: &str, default: &str) -> String {
        self.0
            .get(name)
            .map(|value| value.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

pub struct EntityRevisionPayload {
    pub author_id: i32,
    pub repository_id: i32,
    pub fields: HashMap<String, String>,
}

impl EntityRevisionPayload {
    pub fn new(author_id: i32, repository_id: i32, fields: HashMap<String, String>) -> Self {
        Self {
            author_id,
            repository_id,
            fields,
        }
    }

    pub async fn save<'a, E>(&self, executor: E) -> Result<Uuid, operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator)
                    VALUES (0, 'entityRevision')
            "#,
        )
        .execute(&mut transaction)
        .await?;

        let entity_revision_id = sqlx::query!(r#"SELECT LAST_INSERT_ID() as id"#)
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        sqlx::query!(
            r#"
                INSERT INTO entity_revision (id, author_id, repository_id, date)
                    VALUES (?, ?, ?, ?)
            "#,
            entity_revision_id,
            self.author_id,
            self.repository_id,
            DateTime::now(),
        )
        .execute(&mut transaction)
        .await?;

        for (field, value) in &self.fields {
            let field_snake_case = field.to_case(Case::Snake);

            sqlx::query!(
                r#"
                    INSERT INTO entity_revision_field (field, value, entity_revision_id)
                        VALUES (?, ?, ?)
                "#,
                field_snake_case,
                value,
                entity_revision_id,
            )
            .execute(&mut transaction)
            .await?;
        }

        let uuid =
            EntityRevision::fetch_via_transaction(entity_revision_id, &mut transaction).await?;

        transaction.commit().await?;

        Ok(uuid)
    }
}
