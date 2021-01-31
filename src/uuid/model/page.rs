use serde::Serialize;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError};
use crate::datetime::DateTime;
use crate::format_alias;
use crate::instance::Instance;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub instance: Instance,
    pub current_revision_id: Option<i32>,
    pub revision_ids: Vec<i32>,
    pub date: DateTime,
    pub license_id: i32,
}

impl Page {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let page = sqlx::query!(
            r#"
                SELECT u.trashed, i.subdomain, p.current_revision_id, p.license_id, r.title
                    FROM page_repository p
                    JOIN uuid u ON u.id = p.id
                    JOIN instance i ON i.id = p.instance_id
                    LEFT JOIN page_revision r ON r.id = p.current_revision_id
                    WHERE p.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;

        let revisions = sqlx::query!(
            r#"SELECT id, date FROM page_revision WHERE page_repository_id = ?"#,
            id
        )
        .fetch_all(pool)
        .await?;

        if revisions.is_empty() {
            Err(UuidError::NotFound)
        } else {
            Ok(Uuid {
                id,
                trashed: page.trashed != 0,
                // TODO:
                alias: format_alias(None, id, page.title.as_deref()),
                concrete_uuid: ConcreteUuid::Page(Page {
                    __typename: "Page".to_string(),
                    instance: page
                        .subdomain
                        .parse()
                        .map_err(|_| UuidError::InvalidInstance)?,
                    current_revision_id: page.current_revision_id,
                    revision_ids: revisions
                        .iter()
                        .rev()
                        .map(|revision| revision.id as i32)
                        .collect(),
                    date: revisions[0].date.into(),
                    license_id: page.license_id,
                }),
            })
        }
    }
}
