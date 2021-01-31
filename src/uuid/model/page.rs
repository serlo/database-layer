use async_trait::async_trait;
use futures::join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};
use crate::database::Executor;
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

macro_rules! fetch_one_page {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT u.trashed, i.subdomain, p.current_revision_id, p.license_id, r.title
                    FROM page_repository p
                    JOIN uuid u ON u.id = p.id
                    JOIN instance i ON i.id = p.instance_id
                    LEFT JOIN page_revision r ON r.id = p.current_revision_id
                    WHERE p.id = ?
            "#,
            $id
        )
        .fetch_one($executor)
    };
}

macro_rules! fetch_all_revisions {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"SELECT id, date FROM page_revision WHERE page_repository_id = ?"#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! to_page {
    ($id: expr, $page: expr, $revisions: expr) => {{
        let page = $page.map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let revisions = $revisions?;

        if revisions.is_empty() {
            Err(UuidError::NotFound)
        } else {
            Ok(Uuid {
                id: $id,
                trashed: page.trashed != 0,
                // TODO:
                alias: format_alias(None, $id, page.title.as_deref()),
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
    }};
}

#[async_trait]
impl UuidFetcher for Page {
    async fn fetch(id: i32, pool: &MySqlPool) -> Result<Uuid, UuidError> {
        let page = fetch_one_page!(id, pool);
        let revisions = fetch_all_revisions!(id, pool);

        let (page, revisions) = join!(page, revisions);

        to_page!(id, page, revisions)
    }

    async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Uuid, UuidError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let page = fetch_one_page!(id, &mut transaction).await;
        let revisions = fetch_all_revisions!(id, &mut transaction).await;

        let result = to_page!(id, page, revisions);

        transaction.commit().await?;

        result
    }
}
