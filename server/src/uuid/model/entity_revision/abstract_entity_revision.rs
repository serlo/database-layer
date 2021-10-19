use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::UuidError;
use crate::datetime::DateTime;
use crate::uuid::EntityType;

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

#[derive(Debug, Deserialize, Serialize)]
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
