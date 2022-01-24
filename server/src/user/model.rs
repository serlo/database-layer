use crate::database::Executor;
use crate::datetime::DateTime;
use crate::user::messages::{potential_spam_users_query, user_activity_by_type_query, user_delete_bots_mutation, user_set_description_mutation, user_set_email_mutation};
use std::env;

pub struct User {}

impl User {
    pub async fn fetch_active_authors<'a, E>(executor: E) -> Result<Vec<i32>, sqlx::Error>
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

    pub async fn fetch_active_reviewers<'a, E>(executor: E) -> Result<Vec<i32>, sqlx::Error>
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
    ) -> Result<user_activity_by_type_query::Output, sqlx::Error>
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
                            WHEN event_id in (8,9,14,16) THEN "comments"
                            ELSE "taxonomy"
                        END AS type
                        FROM event_log
                        WHERE actor_id = ?
                            AND event_id IN (5,6,11,8,9,14,16,1,2,12,15,17)
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

        Ok(user_activity_by_type_query::Output {
            edits: find_counts("edits"),
            reviews: find_counts("reviews"),
            comments: find_counts("comments"),
            taxonomy: find_counts("taxonomy"),
        })
    }

    pub async fn delete_bot<'a, E>(
        payload: &user_delete_bots_mutation::Payload,
        executor: E,
    ) -> Result<Vec<String>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;
        let mut email_hashes: Vec<String> = Vec::new();

        for bot_id in &payload.bot_ids {
            let result = sqlx::query!("select email from user where id = ?", bot_id)
                .fetch_optional(&mut transaction)
                .await?
                .map(|user| user.email);

            if let Some(email) = result {
                email_hashes.push(format!("{:x}", md5::compute(email.as_bytes())).to_string());
            }

            sqlx::query!(
                r#"DELETE FROM uuid WHERE id = ? AND discriminator = 'user'"#,
                bot_id,
            )
            .execute(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(email_hashes)
    }

    pub async fn potential_spam_users<'a, E>(
        payload: &potential_spam_users_query::Payload,
        executor: E,
    ) -> Result<Vec<i32>, sqlx::Error>
    where
        E: Executor<'a>,
    {
        println!("{:?}", payload.after);
        Ok(sqlx::query!(
            r#"
                select user.id
                from user
                where
                    user.description is not null
                    and user.description != "NULL"
                    and (? is null or user.id < ?)
                order by user.id desc
                limit ?
            "#,
            payload.after,
            payload.after,
            payload.first,
        )
        .fetch_all(executor)
        .await?
        .into_iter()
        .map(|x| x.id as i32)
        .collect())
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

    pub async fn set_description<'a, E>(
        payload: &user_set_description_mutation::Payload,
        executor: E,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'a>,
    {
        sqlx::query!(
            "update user set description = ? where id = ?",
            payload.description,
            payload.user_id
        )
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn set_email<'a, E>(
        payload: &user_set_email_mutation::Payload,
        executor: E,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'a>,
    {
        sqlx::query!(
            "update user set email = ? where id = ?",
            payload.email,
            payload.user_id
        )
        .execute(executor)
        .await?;
        Ok(())
    }
}
