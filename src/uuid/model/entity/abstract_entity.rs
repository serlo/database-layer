use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub enum EntityType {
    Applet,
    Article,
    Course,
    CoursePage,
    Event,
    #[serde(rename(deserialize = "text-exercise"))]
    Exercise,
    #[serde(rename(deserialize = "text-exercise-group"))]
    ExerciseGroup,
    #[serde(rename(deserialize = "grouped-text-exercise"))]
    GroupedExercise,
    #[serde(rename(deserialize = "text-solution"))]
    Solution,
    Video,
}

impl std::str::FromStr for EntityType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}
