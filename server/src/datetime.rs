//! Provides a custom `DateTime` `newtype` that deals with timestamp inconsistencies in our database.
//!
//! The timestamps in our database are persisted as UTC but are actually Europe/Berlin. So we need
//! to manipulate the timestamps accordingly. See [`DateTime`].
use std::fmt;

use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::Berlin;
use serde::{Serialize, Serializer};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;

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
#[derive(Debug, Eq, PartialEq)]
pub struct DateTime(chrono::DateTime<Utc>);

impl DateTime {
    pub fn now() -> Self {
        let current_datetime = Utc::now();
        DateTime(current_datetime)
    }

    pub fn ymd(year: i32, month: u32, date: u32) -> Self {
        DateTime(Utc.ymd(year, month, date).and_hms(0, 0, 0))
    }

    pub fn signed_duration_since(&self, rhs: DateTime) -> Duration {
        self.0.signed_duration_since(rhs.0)
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
        serializer.serialize_str(&format!("{}", self))
    }
}

impl sqlx::Type<MySql> for DateTime {
    fn type_info() -> MySqlTypeInfo {
        chrono::DateTime::<Utc>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, MySql> for DateTime {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let datetime = self.0;
        let datetime = Berlin.from_utc_datetime(&datetime.naive_utc());
        let datetime = datetime.naive_local();
        let datetime = Utc.from_utc_datetime(&datetime);
        datetime.encode_by_ref(buf)
    }
}
