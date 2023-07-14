use crate::datetime::DateTime;
use crate::operation;
use crate::user::messages::{
    potential_spam_users_query, user_activity_by_type_query, user_add_role_mutation,
    user_create_mutation, user_delete_bots_mutation, user_delete_regular_users_mutation,
    user_remove_role_mutation, user_set_description_mutation, user_set_email_mutation,
    users_by_role_query,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;

pub struct User {}

impl User {
    pub async fn fetch_active_authors<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        acquire_from: A,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let mut connection = acquire_from.acquire().await?;
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
        .fetch_all(&mut *connection)
        .await?;
        Ok(user_ids.iter().map(|user| user.id as i32).collect())
    }

    pub async fn fetch_active_reviewers<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        acquire_from: A,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let mut connection = acquire_from.acquire().await?;
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
            .fetch_all(&mut *connection)
            .await?;
        Ok(user_ids.iter().map(|user| user.id as i32).collect())
    }

    pub async fn fetch_activity_by_type<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        user_id: i32,
        acquire_from: A,
    ) -> Result<user_activity_by_type_query::Output, sqlx::Error> {
        let mut connection = acquire_from.begin().await?;
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
        .fetch_all(&mut *connection)
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

    pub async fn add_role<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &user_add_role_mutation::Payload,
        acquire_from: A,
    ) -> Result<(), operation::Error> {
        let mut transaction = acquire_from.begin().await?;

        let role_id = Self::role_name_to_id(&payload.role_name, &mut transaction).await?;

        let user_id = sqlx::query!(
            r#"
                SELECT id
                FROM user
                WHERE username = ?
            "#,
            payload.username
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: "This user does not exist.".to_string(),
        })?
        .id;

        let response = sqlx::query!(
            r#"
                SELECT role_id
                FROM role_user
                WHERE user_id = ? AND role_id = ?
            "#,
            user_id,
            role_id,
        )
        .fetch_optional(&mut *transaction)
        .await?;

        if response.is_some() {
            return Ok(());
        }

        sqlx::query!(
            r#"
                INSERT INTO role_user (user_id, role_id)
                VALUES (?, ?)
            "#,
            user_id,
            role_id
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn create<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &user_create_mutation::Payload,
        acquire_from: A,
    ) -> Result<i32, operation::Error> {
        let default_role_id: i32 = 2;

        if payload.username.len() > 32 {
            return Err(operation::Error::BadRequest {
                reason: "Username can\'t be longer than 32 characters.".to_string(),
            });
        }

        if payload.email.len() > 254 {
            return Err(operation::Error::BadRequest {
                reason: "Email can\'t be longer than 254 characters.".to_string(),
            });
        }

        if payload.username.trim().is_empty() {
            return Err(operation::Error::BadRequest {
                reason: "Username can\'t be empty.".to_string(),
            });
        }

        if payload.password.len() > 50 {
            return Err(operation::Error::BadRequest {
                reason: "Password can\'t be longer than 50 characters.".to_string(),
            });
        }

        let mut transaction = acquire_from.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (discriminator)
                VALUES ('user')
            "#,
        )
        .execute(&mut *transaction)
        .await?;

        let user_id = sqlx::query!(
            r#"
                SELECT LAST_INSERT_ID() AS id
                FROM uuid
            "#,
        )
        .fetch_one(&mut *transaction)
        .await?
        .id;

        let token: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        sqlx::query!(
            r#"
                INSERT INTO user (id, email, username, password, date, token)
                VALUES (?, ?, ?, ?, ?, ?)
            "#,
            user_id,
            payload.email,
            payload.username,
            payload.password,
            DateTime::now(),
            token.to_lowercase(),
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO role_user (user_id, role_id)
                VALUES (?, ?)
            "#,
            user_id,
            default_role_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(user_id as i32)
    }

    pub async fn delete_bot<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &user_delete_bots_mutation::Payload,
        acquire_from: A,
    ) -> Result<Vec<String>, sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;
        let mut email_hashes: Vec<String> = Vec::new();

        for bot_id in &payload.bot_ids {
            let result = sqlx::query!("select email from user where id = ?", bot_id)
                .fetch_optional(&mut *transaction)
                .await?
                .map(|user| user.email);

            if let Some(email) = result {
                email_hashes.push(format!("{:x}", md5::compute(email.as_bytes())).to_string());
            }

            sqlx::query!(
                r#"DELETE FROM uuid WHERE id = ? AND discriminator = 'user'"#,
                bot_id,
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(email_hashes)
    }

    pub async fn delete_regular_user<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &user_delete_regular_users_mutation::Payload,
        acquire_from: A,
    ) -> Result<(), operation::Error> {
        let deleted_user_id: i32 = 4;

        if payload.user_id == deleted_user_id {
            return Err(operation::Error::BadRequest {
                reason: "You cannot delete the user Deleted.".to_string(),
            });
        }

        let mut transaction = acquire_from.begin().await?;

        sqlx::query!(r#"select * from user where id = ?"#, payload.user_id)
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or(operation::Error::BadRequest {
                reason: "The requested user does not exist.".to_string(),
            })?;

        sqlx::query!(
            r#"update ad set author_id = ? where author_id = ?"#,
            deleted_user_id,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"update blog_post set author_id = ? where author_id = ?"#,
            deleted_user_id,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"update comment set author_id = ? where author_id = ?"#,
            deleted_user_id,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"delete from comment_vote where user_id = ?"#,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"update entity_revision set author_id = ? where author_id = ?"#,
            deleted_user_id,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"update event_log set actor_id = ? where actor_id = ?"#,
            deleted_user_id,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(r#"delete from flag where reporter_id = ?"#, payload.user_id)
            .execute(&mut *transaction)
            .await?;

        sqlx::query!(
            r#"delete from notification where user_id = ?"#,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"update page_revision set author_id = ? where author_id = ?"#,
            deleted_user_id,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"delete from role_user where user_id = ?"#,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"delete from subscription where user_id = ?"#,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"delete from subscription where uuid_id = ?"#,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"delete from uuid where id = ? and discriminator = 'user'"#,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn potential_spam_users<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &potential_spam_users_query::Payload,
        acquire_from: A,
    ) -> Result<Vec<i32>, sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;
        let result = sqlx::query!(
            r#"
                SELECT id
                FROM (
                    SELECT user.id AS id, MAX(role_user.role_id) AS role_id
                    FROM user
                    LEFT JOIN role_user ON user.id = role_user.user_id
                    WHERE user.description IS NOT NULL
                        AND user.description != "NULL"
                        AND (? IS NULL OR user.id < ?)
                    GROUP BY user.id
                ) A
                WHERE (role_id IS NULL OR role_id <= 2)
                ORDER BY id DESC
                LIMIT ?
            "#,
            payload.after,
            payload.after,
            payload.first,
        )
        .fetch_all(&mut *transaction)
        .await?
        .into_iter();

        let mut ids: Vec<i32> = Vec::new();
        for item in result {
            let activity = User::fetch_activity_by_type(item.id as i32, &mut *transaction).await?;
            if activity.edits <= 5 {
                ids.push(item.id as i32);
            }
        }
        Ok(ids)
    }

    pub async fn remove_role<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &user_remove_role_mutation::Payload,
        acquire_from: A,
    ) -> Result<(), operation::Error> {
        let mut transaction = acquire_from.begin().await?;

        let role_id = Self::role_name_to_id(&payload.role_name, &mut transaction).await?;

        let user_id = sqlx::query!(
            r#"
                SELECT id
                FROM user
                WHERE username = ?
            "#,
            payload.username
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: "This user does not exist.".to_string(),
        })?
        .id;

        sqlx::query!(
            "DELETE role_user
            FROM role_user
            WHERE user_id = ?
                AND role_id = ?",
            user_id,
            role_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn users_by_role<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &users_by_role_query::Payload,
        acquire_from: A,
    ) -> Result<Vec<i32>, operation::Error> {
        if payload.first > 10000 {
            return Err(operation::Error::BadRequest {
                reason: "The parameter first is not allowed to be greater than 10,000.".to_string(),
            });
        }
        let mut transaction = acquire_from.begin().await?;
        let role_id = Self::role_name_to_id(&payload.role_name, &mut transaction).await?;
        Ok(sqlx::query!(
            r#"
                    SELECT user_id
                    FROM role_user
                    WHERE role_id = ?
                        AND (? IS NULL OR user_id > ?)
                    ORDER BY user_id
                    LIMIT ?
                "#,
            role_id,
            payload.after,
            payload.after,
            payload.first,
        )
        .fetch_all(&mut *transaction)
        .await?
        .into_iter()
        .map(|x| x.user_id as i32)
        .collect())
    }

    pub async fn set_description<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &user_set_description_mutation::Payload,
        acquire_from: A,
    ) -> Result<(), operation::Error> {
        let mut connection = acquire_from.acquire().await?;
        if payload.description.len() >= 64 * 1024 {
            return Err(operation::Error::BadRequest {
                reason: "description is too long".to_string(),
            });
        }

        sqlx::query!(
            "update user set description = ? where id = ?",
            payload.description,
            payload.user_id
        )
        .execute(&mut *connection)
        .await?;
        Ok(())
    }

    pub async fn set_email<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &user_set_email_mutation::Payload,
        acquire_from: A,
    ) -> Result<String, sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;

        let username = sqlx::query!("select username from user where id = ?", payload.user_id)
            .fetch_one(&mut *transaction)
            .await?
            .username;
        sqlx::query!(
            "update user set email = ? where id = ?",
            payload.email,
            payload.user_id
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(username)
    }

    async fn role_name_to_id<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        name: &str,
        acquire_from: A,
    ) -> Result<i32, operation::Error> {
        let mut transaction = acquire_from.begin().await?;
        Ok(sqlx::query!(
            r#"
                SELECT id
                FROM role
                WHERE name = ?
            "#,
            name
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(operation::Error::BadRequest {
            reason: "This role does not exist.".to_string(),
        })?
        .id)
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
