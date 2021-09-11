use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::{Error, MySql};

use crate::database::Executor;
use std::fmt::Formatter;

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

impl Instance {
    pub async fn fetch_id<'a, E>(&self, executor: E) -> Result<i32, Error>
    where
        E: Executor<'a>,
    {
        Ok(
            sqlx::query!("SELECT id FROM instance WHERE subdomain = ?", self)
                .fetch_one(executor)
                .await?
                .id,
        )
    }
}

impl std::str::FromStr for Instance {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let decoded = serde_json::to_value(self).unwrap();
        let decoded = decoded.as_str().unwrap();
        write!(f, "{}", decoded)
    }
}

impl sqlx::Type<MySql> for Instance {
    fn type_info() -> MySqlTypeInfo {
        str::type_info()
    }
}
impl<'q> sqlx::Encode<'q, MySql> for Instance {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let decoded = format!("{}", self);
        decoded.encode_by_ref(buf)
    }
}
