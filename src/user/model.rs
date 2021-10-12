use crate::database::Executor;
use crate::datetime::DateTime;
use crate::user::messages::user_activity_by_type_query;
use crate::user::messages::user_delete_bots_mutation;
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

        Ok(user_activity_by_type_query::Output {
            edits: find_counts("edits"),
            reviews: find_counts("reviews"),
            comments: find_counts("comments"),
            taxonomy: find_counts("taxonomy"),
        })
    }

    // maybe always use delete_user since performace should not matter here?
    pub async fn delete_bot<'a, E>(
        payload: &user_delete_bots_mutation::Payload,
        executor: E,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for user_id in &payload.user_ids {
            sqlx::query!(
                r#"
                    DELETE FROM uuid WHERE id = ? AND discriminator = 'user';
                    DELETE FROM subscription WHERE uuid_id = ?;
                    DELETE FROM comment WHERE author_id = ?;
                "#,
                user_id,
                user_id,
                user_id
            )
            .execute(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    /*
    pub async fn delete_user<'a, E>(
        payload: &user_delete_users_mutation::Payload,
        executor: E,
    ) -> Result<(), operation::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        for user_id in &payload.user_ids {
            sqlx::query!(
                r#"
                    UPDATE ad SET author_id = 4 WHERE author_id = ?;
                    UPDATE blog_post SET author_id = 4 WHERE author_id = ?;
                    UPDATE comment SET author_id = 4 WHERE author_id = ?;
                    DELETE FROM comment_vote WHERE user_id = ?;
                    UPDATE entity_revision SET author_id = 4 WHERE author_id = ?;
                    UPDATE event_log SET actor_id = 4 WHERE actor_id = ?;
                    DELETE FROM flag WHERE reporter_id = ?;
                    DELETE FROM notification WHERE user_id = ?;
                    UPDATE page_revision SET author_id = 4 WHERE author_id = ?;
                    DELETE FROM role_user WHERE user_id = ?;
                    DELETE FROM subscription WHERE uuid_id = ?;
                    DELETE FROM uuid WHERE id = ? AND discriminator = 'user';
                "#,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
                user_id,
            )
            .execute(&mut transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }*/

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
