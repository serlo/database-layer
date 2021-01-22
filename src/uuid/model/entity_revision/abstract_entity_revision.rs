use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEntityRevision {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EntityRevisionType,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub date: String,
    pub author_id: i32,
    pub repository_id: i32,
    pub changes: String,

    #[serde(skip)]
    pub fields: EntityRevisionFields,
}

#[derive(Deserialize, Serialize)]
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
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}

pub struct EntityRevisionFields(pub HashMap<String, String>);

impl EntityRevisionFields {
    pub fn get_or(&self, name: &str, default: &str) -> String {
        self.0
            .get(name)
            .map(|value| value.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}
