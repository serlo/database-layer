use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use super::navigation_child::NavigationChild;
use crate::database::Executor;
use crate::instance::Instance;

#[derive(Serialize)]
pub struct Navigation {
    pub data: Vec<NavigationChild>,
    pub instance: Instance,
}

#[derive(Error, Debug)]
pub enum NavigationError {
    #[error("Navigation cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for NavigationError {
    fn from(inner: sqlx::Error) -> Self {
        NavigationError::DatabaseError { inner }
    }
}

macro_rules! fetch_all_pages {
    ($instance: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                    SELECT p.id
                        FROM navigation_page p
                        JOIN navigation_container c ON c.id = p.container_id
                        JOIN instance i ON i.id = c.instance_id
                        JOIN type t ON t.id = c.type_id
                        WHERE i.subdomain = ? AND t.name = 'default' AND p.parent_id IS NULL
                        ORDER BY p.position, p.id
                "#,
            $instance
        )
        .fetch_all($executor)
    };
}

impl Navigation {
    pub async fn fetch(
        instance: Instance,
        pool: &MySqlPool,
    ) -> Result<Navigation, NavigationError> {
        let pages = fetch_all_pages!(instance, pool).await?;

        let ids: Vec<i32> = pages.iter().map(|page| page.id).collect();
        let data = NavigationChild::bulk_fetch(&ids, pool).await?;

        Ok(Navigation { data, instance })
    }

    #[allow(dead_code)]
    pub async fn fetch_via_transaction<'a, E>(
        instance: Instance,
        executor: E,
    ) -> Result<Navigation, NavigationError>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        let pages = fetch_all_pages!(instance, &mut transaction).await?;

        let ids: Vec<i32> = pages.iter().map(|page| page.id).collect();
        let data = NavigationChild::bulk_fetch_via_transaction(&ids, &mut transaction).await?;

        transaction.commit().await?;

        Ok(Navigation { data, instance })
    }
}
