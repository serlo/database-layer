//! Provides a custom `DateTime` `newtype` that deals with timestamp inconsistencies in our database.
//!
//! The timestamps in our database are persisted as UTC but are actually Europe/Berlin. So we need
//! to manipulate the timestamps accordingly. See [`DateTime`].
use std::fmt;
use std::str::FromStr;

use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Berlin;
use serde::{Deserialize, Serialize, Serializer};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;

use crate::operation;
/// Represents a timestamp in our database.
///
/// # Constructing a `DateTime`
///
/// To construct a `DateTime` representing the current date, use [`DateTime::now`]:
///
/// ```rust
/// use server::datetime::DateTime;
///
/// let current_datetime = DateTime::now();
/// ```
///
/// Timestamps from the database can be converted using the `From` trait:
///
/// ```rust
/// use server::datetime::DateTime;
///
/// # async fn fetch(pool: &sqlx::MySqlPool) -> Result<(), sqlx::Error> {
/// let event = sqlx::query!(r#"SELECT date FROM event_log WHERE id = 1"#)
///     .fetch_one(pool)
///     .await?;
///
/// let datetime: DateTime = event.date.into();
/// # Ok(())
/// # }
/// ```
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct DateTime(chrono::DateTime<Utc>);

impl DateTime {
    pub fn now() -> Self {
        let current_datetime = Utc::now();
        DateTime(current_datetime)
    }

    pub fn ymd(year: i32, month: u32, date: u32) -> Self {
        // TODO: Proper handling of unwrap here...
        DateTime(
            Utc.with_ymd_and_hms(year, month, date, 0, 0, 0)
                .latest()
                .unwrap(),
        )
    }

    pub fn signed_duration_since(&self, rhs: DateTime) -> Duration {
        self.0.signed_duration_since(rhs.0)
    }

    pub fn parse_after_option(input: Option<&String>) -> operation::Result<DateTime> {
        Ok(input
            .map(|s| s.parse())
            .transpose()?
            .unwrap_or(DateTime::now()))
    }
}

impl FromStr for DateTime {
    type Err = operation::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let date =
            chrono::DateTime::parse_from_rfc3339(s).map_err(|_| operation::Error::BadRequest {
                reason: "The date format should be YYYY-MM-DDThh:mm:ss{Timezone}".to_string(),
            })?;
        Ok(DateTime(chrono::DateTime::<Utc>::from(date)))
    }
}

impl From<chrono::DateTime<Utc>> for DateTime {
    fn from(datetime: chrono::DateTime<Utc>) -> Self {
        let datetime = datetime.naive_utc();
        let datetime = Berlin.from_local_datetime(&datetime).unwrap();
        let datetime = datetime.naive_utc();
        let datetime = Utc.from_utc_datetime(&datetime);
        DateTime(datetime)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let datetime = Berlin.from_utc_datetime(&self.0.naive_utc());
        f.write_str(&datetime.to_rfc3339())
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}

impl sqlx::Type<MySql> for DateTime {
    fn type_info() -> MySqlTypeInfo {
        <sqlx::types::chrono::DateTime<Utc> as sqlx::Type<MySql>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, MySql> for DateTime {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <chrono::DateTime<chrono::Utc> as sqlx::Encode<'_, MySql>>::encode_by_ref(
            &Utc.from_utc_datetime(&Berlin.from_utc_datetime(&self.0.naive_utc()).naive_local()),
            buf,
        )
    }
}
