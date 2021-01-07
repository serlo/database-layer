use crate::uuid::model::page::Page;
use crate::uuid::model::user::User;
use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Uuid {
    User(User),
    Page(Page),
}

impl Uuid {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Uuid> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(&*pool)
            .await?;
        match uuid.discriminator.as_str() {
            "page" => Ok(Uuid::Page(Page::find_by_id(id, pool).await?)),
            "user" => Ok(Uuid::User(User::find_by_id(id, pool).await?)),
            _ => {
                panic!("TODO")
            }
        }
    }
}
