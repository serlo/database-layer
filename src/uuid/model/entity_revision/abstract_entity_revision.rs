use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::UuidError;
use crate::datetime::DateTime;

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
    #[serde(rename(serialize = "AppletRevision"))]
    Applet,
    #[serde(rename(serialize = "ArticleRevision"))]
    Article,
    #[serde(rename(serialize = "CourseRevision"))]
    Course,
    #[serde(rename(serialize = "CoursePageRevision"))]
    CoursePage,
    #[serde(rename(serialize = "EventRevision"))]
    Event,
    #[serde(rename(serialize = "ExerciseRevision", deserialize = "text-exercise"))]
    Exercise,
    #[serde(rename(
        serialize = "ExerciseGroupRevision",
        deserialize = "text-exercise-group"
    ))]
    ExerciseGroup,
    #[serde(rename(
        serialize = "GroupedExerciseRevision",
        deserialize = "grouped-text-exercise"
    ))]
    GroupedExercise,
    #[serde(rename(serialize = "SolutionRevision", deserialize = "text-solution"))]
    Solution,
    #[serde(rename(serialize = "VideoRevision"))]
    Video,
}

impl std::str::FromStr for EntityRevisionType {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string())).map_err(|_e| {
            UuidError::UnsupportedEntityRevisionType {
                name: s.to_string(),
            }
        })
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
