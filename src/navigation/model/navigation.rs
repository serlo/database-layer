use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use super::navigation_child::{NavigationChild, NavigationChildError};
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

impl Navigation {
    pub async fn fetch(
        instance: Instance,
        pool: &MySqlPool,
    ) -> Result<Navigation, NavigationError> {
        let pages = sqlx::query!(
            r#"
                SELECT p.id
                    FROM navigation_page p
                    JOIN navigation_container c ON c.id = p.container_id
                    JOIN instance i ON i.id = c.instance_id
                    JOIN type t ON t.id = c.type_id
                    WHERE i.subdomain = ? AND t.name = 'default' AND p.parent_id IS NULL
                    ORDER BY p.position, p.id
            "#,
            instance
        )
        .fetch_all(pool)
        .await?;

        let mut data = Vec::with_capacity(pages.len());

        for page in pages.iter() {
            match NavigationChild::fetch(page.id, pool).await {
                Ok(navigation_child) => data.push(navigation_child),
                Err(error) => match error {
                    NavigationChildError::DatabaseError { inner } => return Err(inner.into()),
                    NavigationChildError::NotVisible => {}
                    NavigationChildError::InvalidRoute => {}
                    NavigationChildError::MissingRequiredRouteParameter => {}
                    NavigationChildError::Unsupported => {}
                },
            }
        }

        Ok(Navigation { data, instance })
    }
}
