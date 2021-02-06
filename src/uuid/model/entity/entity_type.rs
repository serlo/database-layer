use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;

use super::UuidError;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RawEntityType {
    Applet,
    Article,
    Course,
    CoursePage,
    Event,
    #[serde(rename = "text-exercise")]
    Exercise,
    #[serde(rename = "text-exercise-group")]
    ExerciseGroup,
    #[serde(rename = "grouped-text-exercise")]
    GroupedExercise,
    #[serde(rename = "text-solution")]
    Solution,
    Video,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum EntityType {
    Applet,
    Article,
    Course,
    CoursePage,
    Event,
    Exercise,
    ExerciseGroup,
    GroupedExercise,
    Solution,
    Video,
}

impl From<RawEntityType> for EntityType {
    fn from(raw_entity_type: RawEntityType) -> Self {
        match raw_entity_type {
            RawEntityType::Applet => Self::Applet,
            RawEntityType::Article => Self::Article,
            RawEntityType::Course => Self::Course,
            RawEntityType::CoursePage => Self::CoursePage,
            RawEntityType::Event => Self::Event,
            RawEntityType::Exercise => Self::Exercise,
            RawEntityType::ExerciseGroup => Self::ExerciseGroup,
            RawEntityType::GroupedExercise => Self::GroupedExercise,
            RawEntityType::Solution => Self::Solution,
            RawEntityType::Video => Self::Video,
        }
    }
}

impl From<EntityType> for RawEntityType {
    fn from(raw_entity_type: EntityType) -> Self {
        match raw_entity_type {
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

impl std::str::FromStr for EntityType {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value::<RawEntityType>(serde_json::value::Value::String(s.to_string()))
            .map(|raw_entity_type| raw_entity_type.into())
            .map_err(|_| UuidError::UnsupportedEntityType {
                name: s.to_string(),
            })
    }
}

impl sqlx::Type<MySql> for EntityType {
    fn type_info() -> MySqlTypeInfo {
        str::type_info()
    }
}
impl<'q> sqlx::Encode<'q, MySql> for EntityType {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let raw_entity_type: RawEntityType = self.clone().into();
        let decoded = serde_json::to_value(raw_entity_type).unwrap();
        let decoded = decoded.as_str().unwrap();
        decoded.encode_by_ref(buf)
    }
}
