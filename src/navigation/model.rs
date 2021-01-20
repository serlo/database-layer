use anyhow::Result;
use futures::try_join;
use regex::Regex;
use serde::Serialize;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Serialize)]
pub struct Navigation {
    pub data: Vec<NormalizedNavigationChild>,
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
                    ORDER BY p.position, p.id
            "#,
            instance
        )
        .fetch_all(pool)
        .await?;

        let mut data = Vec::with_capacity(pages.len());

        for page in pages.iter() {
            data.push(NavigationChild::find_by_id(page.id, pool).await.await?)
        }

        Ok(Navigation {
            data: data.iter().filter_map(|child| child.normalize()).collect(),
            instance: instance.to_string(),
        })
    }
}

pub struct NavigationChild {
    pub id: i32,
    pub children: Vec<NavigationChild>,
    pub parameters: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum NormalizedNavigationChild {
    Uuid(UuidNavigationChild),
    Url(UrlNavigationChild),
    Container(ContainerNavigationChild),
}

#[derive(Serialize)]
pub struct UuidNavigationChild {
    pub label: String,
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NormalizedNavigationChild>>,
}

#[derive(Serialize)]
pub struct UrlNavigationChild {
    pub label: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NormalizedNavigationChild>>,
}

#[derive(Serialize)]
pub struct ContainerNavigationChild {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NormalizedNavigationChild>>,
}

impl NavigationChild {
    pub async fn find_by_id(
        id: i32,
        pool: &MySqlPool,
        // TODO: `Result` might not be needed here? Need to check if `Future` already handles errors or something
    ) -> Pin<Box<dyn Future<Output = Result<NavigationChild>> + '_>> {
        Box::pin(async move {
            let pages_fut = sqlx::query!(
                r#"
                    SELECT p.id
                        FROM navigation_page p
                        WHERE p.parent_id = ?
                        ORDER BY p.position, p.id
                "#,
                id
            )
            .fetch_all(pool);
            let params_fut = sqlx::query!(
                r#"
                    SELECT name, value FROM
                        (
                        # Level 1
                        SELECT k.name, p.value, p.page_id
                            FROM navigation_parameter p
                            JOIN navigation_parameter_key k ON k.id = p.key_id
                            WHERE p.parent_id IS NULL AND value != ''
                        UNION ALL
                        # Level 2
                        SELECT CONCAT(k1.name, '.', k2.name) as name, p2.value, p2.page_id
                            FROM navigation_parameter p1
                            JOIN navigation_parameter p2 ON p2.parent_id = p1.id
                            JOIN navigation_parameter_key k1 ON k1.id = p1.key_id
                            JOIN navigation_parameter_key k2 ON k2.id = p2.key_id
                            WHERE p1.parent_id IS NULL AND p2.value != ''
                        UNION ALL
                        # Level 3
                        SELECT CONCAT(k1.name, '.', k2.name, '.', k3.name) as name, p3.value, p3.page_id
                            FROM navigation_parameter p1
                            JOIN navigation_parameter p2 ON p2.parent_id = p1.id
                            JOIN navigation_parameter p3 ON p3.parent_id = p2.id
                            JOIN navigation_parameter_key k1 ON k1.id = p1.key_id
                            JOIN navigation_parameter_key k2 ON k2.id = p2.key_id
                            JOIN navigation_parameter_key k3 ON k3.id = p3.key_id
                            WHERE p1.parent_id IS NULL AND p3.value != ''
                        ) u
                        WHERE page_id = ?
                "#,
                id
            )
            .fetch_all(pool);

            let (pages, params) = try_join!(pages_fut, params_fut)?;

            let mut children = Vec::with_capacity(pages.len());

            for page in pages.iter() {
                let ret = NavigationChild::find_by_id(page.id, pool).await.await?;
                children.push(ret);
            }

            let mut parameters: HashMap<String, String> = HashMap::with_capacity(params.len());

            for param in params.iter() {
                if let (Some(name), Some(value)) = (param.name.as_ref(), param.value.as_ref()) {
                    parameters.insert(name.to_string(), value.to_string());
                }
            }

            Ok(NavigationChild {
                id,
                children,
                parameters,
            })
        })
    }

    // TODO: there is probably a better pattern for that (using parse maybe?)
    pub fn normalize(&self) -> Option<NormalizedNavigationChild> {
        if let Some(visible) = self.parameters.get("visible") {
            if visible == "false" {
                return None;
            }
        }

        let children: Vec<NormalizedNavigationChild> = self
            .children
            .iter()
            .filter_map(|child| child.normalize())
            .collect();

        let children = if children.is_empty() {
            None
        } else {
            Some(children)
        };

        if let (Some(label), Some(uri)) = (self.parameters.get("label"), self.parameters.get("uri"))
        {
            return if uri == "#" {
                Some(NormalizedNavigationChild::Container(
                    ContainerNavigationChild {
                        label: label.to_string(),
                        children,
                    },
                ))
            } else {
                let re = Regex::new(r"^/(?P<id>\d+)$").unwrap();
                match re.captures(uri) {
                    Some(captures) => {
                        let id: i32 = captures.name("id").unwrap().as_str().parse().unwrap();
                        Some(NormalizedNavigationChild::Uuid(UuidNavigationChild {
                            label: label.to_string(),
                            id,
                            children,
                        }))
                    }
                    _ => Some(NormalizedNavigationChild::Url(UrlNavigationChild {
                        label: label.to_string(),
                        url: uri.to_string(),
                        children,
                    })),
                }
            };
        }

        if let (Some(label), Some(route)) =
            (self.parameters.get("label"), self.parameters.get("route"))
        {
            return match route.as_str() {
                "blog" => Some(NormalizedNavigationChild::Url(UrlNavigationChild {
                    label: label.to_string(),
                    url: "/blog".to_string(),
                    children,
                })),
                "discussion/discussions" => {
                    Some(NormalizedNavigationChild::Url(UrlNavigationChild {
                        label: label.to_string(),
                        url: "/discussions".to_string(),
                        children,
                    }))
                }
                "discussion/discussions/get" => self
                    .parameters
                    .get("params.id")
                    .and_then(|id| id.parse::<i32>().ok())
                    .map(|id| {
                        NormalizedNavigationChild::Url(UrlNavigationChild {
                            label: label.to_string(),
                            url: format!("/discussions/{}", id),
                            children,
                        })
                    }),
                "event/history/all" => Some(NormalizedNavigationChild::Url(UrlNavigationChild {
                    label: label.to_string(),
                    url: "/event/history".to_string(),
                    children,
                })),
                "page/view" => self
                    .parameters
                    .get("params.page")
                    .and_then(|id| id.parse::<i32>().ok())
                    .map(|id| {
                        NormalizedNavigationChild::Uuid(UuidNavigationChild {
                            label: label.to_string(),
                            id,
                            children,
                        })
                    }),
                "subject/entity" => self.parameters.get("params.subject").map(|subject| {
                    NormalizedNavigationChild::Url(UrlNavigationChild {
                        label: label.to_string(),
                        url: format!("/{}/entity/trash-bin", subject.replace(" ", "%20")),
                        children,
                    })
                }),
                "taxonomy/term/get" => self
                    .parameters
                    .get("params.term")
                    .and_then(|id| id.parse::<i32>().ok())
                    .map(|id| {
                        NormalizedNavigationChild::Uuid(UuidNavigationChild {
                            label: label.to_string(),
                            id,
                            children,
                        })
                    }),
                "taxonomy/term/organize" => self
                    .parameters
                    .get("params.term")
                    .and_then(|id| id.parse::<i32>().ok())
                    .map(|id| {
                        NormalizedNavigationChild::Url(UrlNavigationChild {
                            label: label.to_string(),
                            url: format!("/taxonomy/term/organize/{}", id),
                            children,
                        })
                    }),
                route => {
                    println!("Unhandled route: {}", route);
                    println!("Params: {:?}", self.parameters);
                    None
                }
            };
        }

        None
    }
}
