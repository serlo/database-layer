use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub id: i32,
    pub instance: String,
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
    #[error("License cannot be fetched because it does not exist.")]
    NotFound,
}

impl From<sqlx::Error> for LicenseError {
    fn from(inner: sqlx::Error) -> Self {
        LicenseError::DatabaseError { inner }
    }
}

impl License {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<License, LicenseError> {
        sqlx::query!(
            r#"
                SELECT l.default, l.title, l.url, l.content, l.agreement, l.icon_href, i.subdomain
                    FROM license l
                    JOIN instance i ON i.id = l.instance_id
                    WHERE l.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => LicenseError::NotFound,
            error => error.into(),
        })
        .map(|license| License {
            id,
            instance: license.subdomain,
            default: license.default == Some(1),
            title: license.title,
            url: license.url,
            content: license.content.unwrap_or_else(|| "".to_string()),
            agreement: license.agreement,
            icon_href: license.icon_href.unwrap_or_else(|| "".to_string()),
        })
    }
}
