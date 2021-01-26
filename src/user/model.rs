use std::env;

use sqlx::MySqlPool;
use thiserror::Error;

use crate::datetime::DateTime;

pub struct User {}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Users cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for UserError {
    fn from(inner: sqlx::Error) -> Self {
        UserError::DatabaseError { inner }
    }
}

impl User {
    pub async fn fetch_active_authors(pool: &MySqlPool) -> Result<Vec<i32>, UserError> {
        let user_ids = sqlx::query!(
            r#"
                SELECT u.id
                    FROM user u
                    JOIN event_log e ON u.id = e.actor_id
                    WHERE e.event_id = 5 AND e.date > DATE_SUB(?, Interval 90 day)
                    GROUP BY u.id
                    HAVING count(e.event_id) > 10
            "#,
            Self::now()
        )
        .fetch_all(pool)
        .await?;
        Ok(user_ids.iter().map(|user| user.id as i32).collect())
    }

    pub async fn fetch_active_reviewers(pool: &MySqlPool) -> Result<Vec<i32>, UserError> {
        let user_ids = sqlx::query!(
            r#"
                SELECT u.id
                    FROM event_log e1
                    JOIN event_log e2 ON e1.uuid_id = e2.uuid_id AND (e1.event_id = 6 OR e1.event_id = 11) AND e2.event_id = 5 AND e1.date >= e2.date AND e1.actor_id != e2.actor_id
                    JOIN user u ON u.id = e1.actor_id
                    WHERE e1.date > DATE_SUB(?, Interval 90 day)
                    GROUP BY u.id
                    HAVING count(e1.event_id) > 10
            "#,
            Self::now()
        )
        .fetch_all(pool)
        .await?;
        Ok(user_ids.iter().map(|user| user.id as i32).collect())
    }

    fn now() -> DateTime {
        // In the development database there are no recent edits so we use an old timestamp (2014-01-01).
        // In production, we use the current time.
        let environment = env::var("ENV").unwrap();
        match environment.as_str() {
            "development" => DateTime::ymd(2014, 1, 1),
            _ => DateTime::now(),
        }
    }
}
