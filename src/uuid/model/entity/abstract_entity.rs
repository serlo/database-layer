use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;

use super::UuidError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEntity {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: EntityType,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub instance: String,
    pub date: String,
    pub license_id: i32,
    pub taxonomy_term_ids: Vec<i32>,

    pub current_revision_id: Option<i32>,
    pub revision_ids: Vec<i32>,
}

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
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string())).map_err(|_e| {
            UuidError::UnsupportedEntityType {
                name: s.to_string(),
            }
        })
    }
}

// TODO: with these two `impl`, we can directly pass EntityType into a SQL query. Is there a better way to reverse the rename stuff we did with serde?
impl sqlx::Type<MySql> for EntityType {
    fn type_info() -> MySqlTypeInfo {
        str::type_info()
    }
}
impl<'q> sqlx::Encode<'q, MySql> for EntityType {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let deserialized = match self {
            EntityType::Applet => "applet",
            EntityType::Article => "article",
            EntityType::Course => "course",
            EntityType::CoursePage => "course-page",
            EntityType::Event => "event",
            EntityType::Exercise => "text-exercise",
            EntityType::ExerciseGroup => "text-exercise-group",
            EntityType::GroupedExercise => "grouped-text-exercise",
            EntityType::Solution => "text-solution",
            EntityType::Video => "video",
        };
        deserialized.encode_by_ref(buf)
    }
}
