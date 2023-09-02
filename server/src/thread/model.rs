use serde::Serialize;

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
}
