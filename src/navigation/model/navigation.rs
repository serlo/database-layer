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
        Ok(Navigation {
            data: vec![],
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
