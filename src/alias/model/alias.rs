use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alias {
    pub id: i32,
    pub instance: String,
    pub path: String,
}

impl Alias {
    pub async fn get_alias_data(alias: &String, pool: &MySqlPool) -> Result<Alias> {
        let alias_fut = sqlx::query!(
            "SELECT id FROM comment WHERE uuid_id = ? ORDER BY date DESC",
            alias
        )
        .fetch_all(pool)
        .await?;

        println!("{:?}", alias_fut);

        Ok(Alias {
            id: 0,
            instance: String::from("de"),
            path: String::from("path/path/party"),
        })
    }
}
