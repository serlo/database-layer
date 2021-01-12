use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

pub struct Thread {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadIds {
    pub first_comment_ids: Vec<i32>,
}

impl Thread {
    pub async fn get_thread_ids(id: i32, pool: &MySqlPool) -> Result<ThreadIds> {
        //TODO: legacy function has option to query archived or non-archived comments here, but I don't think the endpoint uses it?

        let result = sqlx::query!(
            "SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC",
            id
        )
        .fetch_all(pool)
        .await?;

        // legacy has code sort this by upvotes and archived state
        // I'm happier with just using the sql ORDER BY
        // frontend handles archived threads diffently anyway

        let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

        Ok(ThreadIds {
            first_comment_ids: first_comment_ids,
        })
    }
}
