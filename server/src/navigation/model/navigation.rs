use std::collections::HashMap;
use std::convert::TryInto;

use serde::Serialize;
use sqlx::MySqlPool;

use super::navigation_child::{
    NavigationChild, NavigationChildError, RawNavigationChild, RawNavigationChildError,
};
use crate::instance::Instance;

#[derive(Serialize)]
pub struct Navigation {
    pub data: Vec<NavigationChild>,
    pub instance: Instance,
}

macro_rules! fetch_all_pages {
    ($instance: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                    SELECT p.id
                        FROM navigation_page p
                        JOIN navigation_container c ON c.id = p.container_id
                        JOIN instance i ON i.id = c.instance_id
                        JOIN type t ON t.id = c.type_id
                        WHERE i.subdomain = ? AND t.name = 'default' AND p.parent_id IS NULL
                        ORDER BY p.position, p.id
                "#,
            $instance
        )
        .fetch_all($executor)
    };
}

macro_rules! handle_raw_navigation_child {
    ($id: expr, $result: expr, $raw_navigation_children: expr, $stack: expr, $ids_dfs: expr) => {
        match $result {
            Ok(navigation_child) => {
                $ids_dfs.push($id);
                for id in navigation_child.children.iter() {
                    $stack.push(*id);
                }
                $raw_navigation_children.insert($id, navigation_child);
            }
            Err(error) => {
                return match error {
                    RawNavigationChildError::DatabaseError { inner } => Err(inner.into()),
                }
            }
        }
    };
}

macro_rules! to_navigation {
    ($instance: expr, $pages: expr, $raw_navigation_children: expr, $ids_dfs: expr) => {{
        let mut navigation_children: HashMap<i32, NavigationChild> =
            HashMap::with_capacity($raw_navigation_children.len());

        while let Some(id) = $ids_dfs.pop() {
            if let Some(raw_navigation_child) = $raw_navigation_children.remove(&id) {
                let children: Vec<NavigationChild> = raw_navigation_child
                    .children
                    .iter()
                    .filter_map(|id| navigation_children.remove(id))
                    .collect();
                match (raw_navigation_child, children).try_into() {
                    Ok(navigation_child) => {
                        navigation_children.insert(id, navigation_child);
                    }
                    Err(error) => match error {
                        NavigationChildError::DatabaseError { inner } => return Err(inner.into()),
                        NavigationChildError::NotVisible => {}
                        NavigationChildError::InvalidRoute => {}
                        NavigationChildError::MissingRequiredRouteParameter => {}
                        NavigationChildError::Unsupported => {}
                    },
                }
            }
        }

        let data = $pages
            .iter()
            .filter_map(|page| navigation_children.remove(&page.id))
            .collect();

        Ok(Navigation {
            data,
            instance: $instance,
        })
    }};
}

impl Navigation {
    pub async fn fetch<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        instance: Instance,
        acquire_from: A,
    ) -> Result<Navigation, sqlx::Error> {
        let mut transaction = acquire_from.begin().await?;

        let pages = fetch_all_pages!(instance, &mut *transaction).await?;

        let mut raw_navigation_children: HashMap<i32, RawNavigationChild> = HashMap::new();
        let mut stack: Vec<i32> = pages.iter().map(|page| page.id).collect();
        let mut ids_dfs: Vec<i32> = Vec::new();

        while let Some(id) = stack.pop() {
            handle_raw_navigation_child!(
                id,
                RawNavigationChild::fetch(id, &mut *transaction).await,
                raw_navigation_children,
                stack,
                ids_dfs
            );
        }

        let result = to_navigation!(instance, pages, raw_navigation_children, ids_dfs);

        transaction.commit().await?;

        result
    }
}
