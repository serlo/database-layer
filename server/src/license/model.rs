use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;
use crate::instance::Instance;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub id: i32,
    pub instance: Instance,
    pub default: bool,
    pub title: String,
    pub url: String,
    pub content: String,
    pub agreement: String,
    pub icon_href: String,
}

#[derive(Error, Debug)]
pub enum LicenseError {
    #[error("License cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("License cannot be fetched because its instance is invalid.")]
    InvalidInstance,
    #[error("License cannot be fetched because it does not exist.")]
    NotFound,
}

impl From<sqlx::Error> for LicenseError {
    fn from(inner: sqlx::Error) -> Self {
        LicenseError::DatabaseError { inner }
    }
}

impl License {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Self, LicenseError> {
        Self::fetch_via_transaction(id, pool).await
    }

    pub async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Self, LicenseError>
    where
        E: Executor<'a>,
    {
        let license = sqlx::query!(
            r#"
                SELECT l.default, l.title, l.url, l.content, l.agreement, l.icon_href, i.subdomain
                    FROM license l
                    JOIN instance i ON i.id = l.instance_id
                    WHERE l.id = ?
            "#,
            id
        )
        .fetch_one(executor)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => LicenseError::NotFound,
            error => error.into(),
        })?;

        Ok(Self {
            id,
            instance: license
                .subdomain
                .parse()
                .map_err(|_| LicenseError::InvalidInstance)?,
            default: license.default == Some(1),
            title: license.title,
            url: license.url,
            content: license.content.unwrap_or_else(|| "".to_string()),
            agreement: license.agreement,
            icon_href: license.icon_href.unwrap_or_else(|| "".to_string()),
        })
    }
}
