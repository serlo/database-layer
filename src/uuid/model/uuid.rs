use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use crate::uuid::model::page::Page;
use crate::uuid::model::page_revision::PageRevision;
use crate::uuid::model::taxonomy_term::TaxonomyTerm;
use crate::uuid::model::user::User;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Uuid {
    Page(Page),
    PageRevision(PageRevision),
    TaxonomyTerm(TaxonomyTerm),
    User(User),
}

impl Uuid {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Uuid> {
        let uuid = sqlx::query!(r#"SELECT discriminator FROM uuid WHERE id = ?"#, id)
            .fetch_one(pool)
            .await?;
        match uuid.discriminator.as_str() {
            "page" => Ok(Uuid::Page(Page::find_by_id(id, pool).await?)),
            "pageRevision" => Ok(Uuid::PageRevision(
                PageRevision::find_by_id(id, pool).await?,
            )),
            "taxonomyTerm" => Ok(Uuid::TaxonomyTerm(
                TaxonomyTerm::find_by_id(id, pool).await?,
            )),
            "user" => Ok(Uuid::User(User::find_by_id(id, pool).await?)),
            _ => Err(anyhow::Error::new(UuidError::UnsupportedDiscriminator {
                id,
                discriminator: uuid.discriminator,
            })),
        }
    }
}

#[derive(Error, Debug)]
pub enum UuidError {
    #[error("UUID {id:?} can't be fetched because is `{discriminator:?}` is not supported.")]
    UnsupportedDiscriminator { id: i32, discriminator: String },
}
