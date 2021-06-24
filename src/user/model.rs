use std::env;

use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;
use crate::datetime::DateTime;

pub struct User {}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Users cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivityByType {
    edits: i32,
    reviews: i32,
    comments: i32,
    taxonomy: i32,
}

impl From<sqlx::Error> for UserError {
    fn from(inner: sqlx::Error) -> Self {
        UserError::DatabaseError { inner }
    }
}

impl User {
    pub async fn fetch_active_authors(pool: &MySqlPool) -> Result<Vec<i32>, UserError> {
        Self::fetch_active_authors_via_transaction(pool).await
    }

    pub async fn fetch_active_authors_via_transaction<'a, E>(
        executor: E,
    ) -> Result<Vec<i32>, UserError>
    where
        E: Executor<'a>,
    {
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
        .fetch_all(executor)
        .await?;
        Ok(user_ids.iter().map(|user| user.id as i32).collect())
    }

    pub async fn fetch_active_reviewers(pool: &MySqlPool) -> Result<Vec<i32>, UserError> {
        Self::fetch_active_reviewers_via_transaction(pool).await
    }

    pub async fn fetch_active_reviewers_via_transaction<'a, E>(
        executor: E,
    ) -> Result<Vec<i32>, UserError>
    where
        E: Executor<'a>,
    {
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
            .fetch_all(executor)
            .await?;
        Ok(user_ids.iter().map(|user| user.id as i32).collect())
    }

    pub async fn fetch_activity_by_type<'a, E>(
        user_id: i32,
        executor: E,
    ) -> Result<UserActivityByType, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let result = sqlx::query!(
            r#"
                SELECT events.type AS event_type, count(*) AS counts
                    FROM (
                        SELECT CASE
                            WHEN event_id = 5 THEN "edits"
                            WHEN event_id in (6,11) THEN "reviews"
                            WHEN event_id in (9,14,16) THEN "comments"
                            ELSE "taxonomy"
                        END AS type
                        FROM event_log
                        WHERE actor_id = ?
                            AND event_id IN (5,6,11,9,14,16,1,2,12,15,17)
                    ) events
                GROUP BY events.type;
            "#,
            user_id
        )
        .fetch_all(executor)
        .await?;

        let find_counts = |event_type: &str| {
            result
                .iter()
                .find(|&row| row.event_type == event_type)
                .map(|row| row.counts)
                .unwrap_or(0) as i32
        };

        Ok(UserActivityByType {
            edits: find_counts("edits"),
            reviews: find_counts("reviews"),
            comments: find_counts("comments"),
            taxonomy: find_counts("taxonomy"),
        })
    }

    fn now() -> DateTime {
        // In the development database there are no recent edits so we use an old timestamp.
        // In production, we use the current time.
        let environment = env::var("ENV").unwrap();
        match environment.as_str() {
            "development" => DateTime::ymd(2014, 1, 1),
            _ => DateTime::now(),
        }
    }
}
