use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
pub struct Navigation {
    pub data: Vec<NavigationChild>,
    pub instance: String,
}

impl Navigation {
    pub async fn find_navigation_by_instance(
        instance: &str,
        pool: &MySqlPool,
    ) -> Result<Navigation> {
        let pages = sqlx::query!(
            r#"
                SELECT p.id
                    FROM navigation_page p
                    JOIN navigation_container c ON c.id = p.container_id
                    JOIN instance i ON i.id = c.instance_id
                    JOIN type t ON t.id = c.type_id
                    WHERE i.subdomain = ? AND t.name = 'default' AND p.parent_id IS NULL
                    ORDER BY p.position
            "#,
            instance
        )
        .fetch_all(pool)
        .await?;

        let mut data = Vec::with_capacity(pages.len());

        for page in pages.iter() {
            let child = NavigationChild::find_by_id(page.id, pool).await?;
            data.push(child)
        }

        Ok(Navigation {
            data,
            instance: String::from(instance),
        })
    }
}

#[derive(Serialize)]
pub struct NavigationChild {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NavigationChild>>,
}

impl NavigationChild {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<NavigationChild> {
        Ok(NavigationChild { id, children: None })
    }
}
