use regex::Regex;
use sqlx::MySqlPool;
use thiserror::Error;

use crate::alias::messages::alias_query;
use crate::database::Executor;
use crate::instance::Instance;
use crate::uuid::{Uuid, UuidError, UuidFetcher};

type Alias = alias_query::Output;

#[derive(Error, Debug)]
pub enum AliasError {
    #[error("Alias cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Alias cannot be fetched because its instance is invalid.")]
    InvalidInstance,
    #[error("Alias is a legacy route.")]
    LegacyRoute,
    #[error("Alias cannot be fetched because it does not exist.")]
    NotFound,
}

impl From<sqlx::Error> for AliasError {
    fn from(inner: sqlx::Error) -> Self {
        match inner {
            sqlx::Error::RowNotFound => AliasError::NotFound,
            _ => AliasError::DatabaseError { inner },
        }
    }
}

impl From<UuidError> for AliasError {
    fn from(error: UuidError) -> Self {
        match error {
            UuidError::DatabaseError { inner } => AliasError::DatabaseError { inner },
            UuidError::InvalidInstance => AliasError::InvalidInstance,
            UuidError::UnsupportedDiscriminator { .. }
            | UuidError::UnsupportedEntityType { .. }
            | UuidError::UnsupportedEntityRevisionType { .. }
            | UuidError::EntityMissingRequiredParent
            | UuidError::NotFound => AliasError::NotFound,
        }
    }
}

pub async fn fetch(path: &str, instance: Instance, pool: &MySqlPool) -> Result<Alias, AliasError> {
    fetch_via_transaction(path, instance, pool).await
}

pub async fn fetch_via_transaction<'a, E>(
    path: &str,
    instance: Instance,
    executor: E,
) -> Result<Alias, AliasError>
where
    E: Executor<'a>,
{
    let path = path.strip_prefix('/').unwrap_or(path);

    if path == "backend"
        || path == "debugger"
        || path == "horizon"
        || path.starts_with("horizon/")
        || path.starts_with("api/")
        || path.is_empty()
        || path == "application"
        || path.starts_with("application/")
        || path.starts_with("attachment/file/")
        || path.starts_with("attachment/upload")
        || path.starts_with("auth/")
        || path.starts_with("authorization/")
        || path == "blog"
        || path.starts_with("blog/view-all/")
        || path.starts_with("blog/view/")
        || path.starts_with("blog/post/")
        || path.starts_with("discussion/")
        || path.starts_with("discussions/")
        || path.starts_with("entities/")
        || path.starts_with("entity/")
        || path.starts_with("event/")
        || path.starts_with("flag/")
        || path.starts_with("license/")
        || path.starts_with("navigation/")
        || path.starts_with("meta/")
        || path.starts_with("ref/")
        || path.starts_with("sitemap/")
        || path.starts_with("notification/")
        || path.starts_with("subscribe/")
        || path.starts_with("unsubscribe/")
        || path.starts_with("subscription/")
        || path.starts_with("subscriptions/")
        || path == "pages"
        || path.starts_with("page/")
        || path.starts_with("related_content/")
        || path == "search"
        || path == "session/gc"
        || path == "spenden"
        || path.starts_with("taxonomies/")
        || path.starts_with("taxonomy/")
        || path == "users"
        || path == "user/me"
        || path == "user/public"
        || path == "user/register"
        || path == "user/settings"
        || path.starts_with("user/remove/")
        || path.starts_with("uuid/")
    {
        return Err(AliasError::LegacyRoute);
    }

    let re = Regex::new(r"^user/profile/(?P<username>.+)$").unwrap();

    let mut transaction = executor.begin().await?;

    let id = if let Some(captures) = re.captures(path) {
        let username = captures.name("username").unwrap().as_str();
        sqlx::query!(
            r#"
                        SELECT id
                            FROM user
                            WHERE username = ?
                    "#,
            username
        )
        .fetch_one(&mut transaction)
        .await?
        .id as i32
    } else {
        let re = Regex::new(r"^(?P<subject>[^/]+/)?(?P<id>\d+)/(?P<title>[^/]*)$").unwrap();
        if let Some(captures) = re.captures(path) {
            captures.name("id").unwrap().as_str().parse().unwrap()
        } else {
            sqlx::query!(
                r#"
                            SELECT a.uuid_id FROM url_alias a
                                JOIN instance i on i.id = a.instance_id
                                WHERE i.subdomain = ? AND a.alias = ?
                                ORDER BY a.timestamp DESC
                        "#,
                instance,
                path
            )
            .fetch_one(&mut transaction)
            .await?
            .uuid_id as i32
        }
    };

    let uuid = Uuid::fetch_via_transaction(id, &mut transaction).await?;

    transaction.commit().await?;

    Ok(alias_query::Output {
        id,
        instance,
        path: uuid.get_alias(),
    })
}
