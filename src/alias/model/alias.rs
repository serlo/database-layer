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
        let re = Regex::new(r"^(?P<subject>[^/]+/)?(?P<id>\d+)/(?P<title>[^/]*)$").unwrap();

        let mut id: Option<i32> = None;

        match re.captures(&path) {
            Some(captures) => {
                // This is an uuid
                id = Some(captures.name("id").unwrap().as_str().parse().unwrap());
            }
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
                id = legacy_alias.first().map(|alias| alias.uuid_id as i32);
            }
        }

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
