use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

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

impl License {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<License> {
        let license_fut = sqlx::query!(
            "
                SELECT l.default, l.title, l.url, l.content, l.agreement, l.icon_href, i.subdomain
                    FROM license l
                    JOIN instance i ON i.id = l.instance_id
                    WHERE l.id = ?
            ",
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(License {
            id,
            instance: license_fut.subdomain,
            default: license_fut.default == Some(1),
            title: license_fut.title,
            url: license_fut.url,
            content: license_fut.content.unwrap_or_else(|| String::from("")),
            agreement: license_fut.agreement,
            icon_href: license_fut.icon_href.unwrap_or_else(|| String::from("")),
        })
    }
}
