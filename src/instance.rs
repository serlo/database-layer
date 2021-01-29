use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Instance {
    De,
    En,
    Es,
    Fr,
    Hi,
    Ta,
}

impl std::str::FromStr for Instance {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}

impl sqlx::Type<MySql> for Instance {
    fn type_info() -> MySqlTypeInfo {
        str::type_info()
    }
}
impl<'q> sqlx::Encode<'q, MySql> for Instance {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let decoded = match self {
            Instance::De => "de",
            Instance::En => "en",
            Instance::Es => "es",
            Instance::Fr => "fr",
            Instance::Hi => "hi",
            Instance::Ta => "ta",
        };
        decoded.encode_by_ref(buf)
    }
}
