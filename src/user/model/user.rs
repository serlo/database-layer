use anyhow::Result;
use sqlx::MySqlPool;

pub struct User {
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub username: String,
    pub date: String,
    pub last_login: Option<String>,
    pub description: Option<String>,
}

impl User {
    pub async fn get_active_author_ids(pool: &MySqlPool) -> Result<Vec<i32>> {
        let results = sqlx::query!(
            r#"
                SELECT user.id as id, count(event_log.event_id) AS edit_counts
                FROM user JOIN event_log on user.id = event_log.actor_id
                WHERE event_log.event_id = 5 and event_log.date > DATE_SUB(Date("2014-01-01"), Interval 90 day)
                GROUP BY user.id
                HAVING edit_counts > 10
            "# // `Date("2014-01-01")` should be replaced with `CURDATE()` in production
        ).fetch_all(pool).await?;
        let user_ids: Vec<i32> = results.iter().map(|user| user.id as i32).collect();
        Ok(user_ids)
    }

    pub async fn get_active_reviewer_ids(pool: &MySqlPool) -> Result<Vec<i32>> {
        let results = sqlx::query!(
            r#"
                SELECT user.id AS id, user.username, count(e1.event_id) AS edit_counts  
                FROM event_log AS e1  
                JOIN event_log AS e2 ON e1.uuid_id = e2.uuid_id AND (e1.event_id = 6 or e1.event_id = 11)
                AND e2.event_id = 5 AND e1.date >= e2.date AND e1.actor_id != e2.actor_id  
                JOIN user ON user.id = e1.actor_id  
                WHERE e1.date > DATE_SUB(Date("2014-01-01"), Interval 90 day)  
                GROUP BY user.id
                HAVING edit_counts > 10
            "# // `Date("2014-01-01")` should be replaced with `CURDATE()` in production
        ).fetch_all(pool).await?;
        let user_ids: Vec<i32> = results.iter().map(|user| user.id as i32).collect();
        Ok(user_ids)
    }
}
