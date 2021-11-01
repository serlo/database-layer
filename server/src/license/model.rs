use sqlx::MySqlPool;

use super::messages::license_query::Output as License;
use crate::database::Executor;
use crate::operation;

pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<License, operation::Error> {
    fetch_via_transaction(id, pool).await
}

pub async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<License, operation::Error>
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
    .await?;

    Ok(License {
        id,
        instance: license.subdomain.parse()?,
        default: license.default == Some(1),
        title: license.title,
        url: license.url,
        content: license.content.unwrap_or_else(|| "".to_string()),
        agreement: license.agreement,
        icon_href: license.icon_href.unwrap_or_else(|| "".to_string()),
    })
}
