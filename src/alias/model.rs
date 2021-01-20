use anyhow::Result;
use regex::Regex;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alias {
    pub id: i32,
    pub instance: String,
    pub path: String,
}

impl Alias {
    pub async fn find_alias_by_path_and_instance(
        path: &str,
        instance: &str,
        pool: &MySqlPool,
    ) -> Result<Option<Alias>> {
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
            return Ok(None);
        }

        let re = Regex::new(r"^user/profile/(?P<username>.+)$").unwrap();
        let id = match re.captures(&path) {
            Some(captures) => {
                let username = captures.name("username").unwrap().as_str();
                let user = sqlx::query!(
                    r#"
                        SELECT id
                            FROM user
                            WHERE username = ?
                    "#,
                    username
                )
                .fetch_all(pool)
                .await?;
                user.first().map(|user| user.id as i32)
            }
            _ => {
                let re = Regex::new(r"^(?P<subject>[^/]+/)?(?P<id>\d+)/(?P<title>[^/]*)$").unwrap();
                match re.captures(&path) {
                    Some(captures) => Some(captures.name("id").unwrap().as_str().parse().unwrap()),
                    _ => {
                        let legacy_alias = sqlx::query!(
                            r#"
                        SELECT a.uuid_id FROM url_alias a
                            JOIN instance i on i.id = a.instance_id
                            WHERE i.subdomain = ? AND a.alias = ?
                            ORDER BY a.timestamp DESC
                    "#,
                            instance,
                            path
                        )
                        .fetch_all(pool)
                        .await?;
                        legacy_alias.first().map(|alias| alias.uuid_id as i32)
                    }
                }
            }
        };

        match id {
            Some(id) => {
                let uuid = Uuid::find_by_id(id, pool).await?;

                Ok(Some(Alias {
                    id,
                    instance: String::from(instance),
                    path: uuid.get_alias(),
                }))
            }
            _ => Ok(None),
        }
    }
}
