use std::env;
use std::time::SystemTime;

use anyhow::Result;
use sqlx::MySqlPool;

pub struct User {}

impl User {
    pub async fn fetch_active_authors(pool: &MySqlPool) -> Result<Vec<i32>> {
        let time = get_mysql_date_string();

        let active_users = sqlx::query!(
            r#"
                SELECT u.id
                    FROM user u
                    JOIN event_log e ON u.id = e.actor_id
                    WHERE e.event_id = 5 AND e.date > DATE_SUB(FROM_UNIXTIME(?), Interval 90 day)
                    GROUP BY u.id
                    HAVING count(e.event_id) > 10
            "#,
            time
        )
        .fetch_all(pool)
        .await?;
        let user_ids: Vec<i32> = active_users.iter().map(|user| user.id as i32).collect();
        Ok(user_ids)
    }

    pub async fn fetch_active_reviewers(pool: &MySqlPool) -> Result<Vec<i32>> {
        let time = get_mysql_date_string();

        let results = sqlx::query!(
            r#"
                SELECT u.id
                    FROM event_log e1
                    JOIN event_log e2 ON e1.uuid_id = e2.uuid_id AND (e1.event_id = 6 OR e1.event_id = 11) AND e2.event_id = 5 AND e1.date >= e2.date AND e1.actor_id != e2.actor_id
                    JOIN user u ON u.id = e1.actor_id
                    WHERE e1.date > DATE_SUB(FROM_UNIXTIME(?), Interval 90 day)
                    GROUP BY u.id
                    HAVING count(e1.event_id) > 10
            "#,
            time
        ).fetch_all(pool).await?;
        let user_ids: Vec<i32> = results.iter().map(|user| user.id as i32).collect();
        Ok(user_ids)
    }
}

fn get_mysql_date_string() -> u64 {
    // In the development database there are no recent edits so we use an old timestamp (2014-01-01).
    // In production, we use the current time.
    let environment = env::var("ENV").unwrap();
    match environment.as_str() {
        "development" => 1388534400,
        _ => {
            let duration = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            duration.as_secs()
        }
    }
}
