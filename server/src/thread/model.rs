use super::messages::set_thread_archived_mutation;
use serde::Serialize;
use sqlx::Row;

use crate::event::SetThreadStateEventPayload;
use crate::operation;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Threads {
    pub first_comment_ids: Vec<i32>,
}

impl Threads {
    pub async fn fetch<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        id: i32,
        acquire_from: A,
    ) -> Result<Self, sqlx::Error> {
        let mut connection = acquire_from.acquire().await?;
        let result = sqlx::query!(
            r#"SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC"#,
            id
        )
        .fetch_all(&mut *connection)
        .await?;

        let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

        Ok(Self { first_comment_ids })
    }

    pub async fn set_archive<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        payload: &set_thread_archived_mutation::Payload,
        acquire_from: A,
    ) -> Result<(), operation::Error> {
        let mut transaction = acquire_from.begin().await?;

        let number_comments = payload.ids.len();
        if number_comments == 0 {
            return Ok(());
        }
        let params = format!("?{}", ", ?".repeat(number_comments - 1));
        let query_str = format!("SELECT id, archived FROM comment WHERE id IN ( {params} )");
        let mut query = sqlx::query(&query_str);
        for id in payload.ids.iter() {
            query = query.bind(id);
        }
        let comments = query.fetch_all(&mut *transaction).await?;
        if comments.len() < number_comments {
            return Err(operation::Error::BadRequest {
                reason: "not all given ids are comments".to_string(),
            });
        }

        let is_archived_after = payload.archived;
        for comment in comments {
            let id: i32 = comment.get("id");
            let is_archived_before: bool = comment.get("archived");
            if is_archived_after != is_archived_before {
                sqlx::query!(
                    r#"
                        UPDATE comment
                            SET archived = ?
                            WHERE id = ?
                    "#,
                    is_archived_after,
                    id
                )
                .execute(&mut *transaction)
                .await?;

                SetThreadStateEventPayload::new(is_archived_after, payload.user_id, id)
                    .save(&mut *transaction)
                    .await
                    .map_err(|error| operation::Error::InternalServerError {
                        error: Box::new(error),
                    })?;
            }
        }

        transaction.commit().await?;

        Ok(())
    }
}
