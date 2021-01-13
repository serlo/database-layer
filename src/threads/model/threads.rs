use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Threads {
    pub first_comment_ids: Vec<i32>,
}

impl Threads {
    pub async fn get_thread_ids(id: i32, pool: &MySqlPool) -> Result<Threads> {
        let result = sqlx::query!(
            "SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC",
            id
        )
        .fetch_all(pool)
        .await?;

        let first_comment_ids: Vec<i32> = result.iter().map(|child| child.id as i32).collect();

        Ok(Threads {
            first_comment_ids: first_comment_ids,
        })
    }
}
