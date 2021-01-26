use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use futures::try_join;
use regex::Regex;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(Serialize)]
#[serde(untagged)]
pub enum NavigationChild {
    Uuid(UuidNavigationChild),
    Url(UrlNavigationChild),
    Container(ContainerNavigationChild),
}

#[derive(Serialize)]
pub struct UuidNavigationChild {
    pub label: String,
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NavigationChild>>,
}

#[derive(Serialize)]
pub struct UrlNavigationChild {
    pub label: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NavigationChild>>,
}

#[derive(Serialize)]
pub struct ContainerNavigationChild {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NavigationChild>>,
}

#[derive(Error, Debug)]
pub enum NavigationChildError {
    #[error("NavigationChild cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("NavigationChild is not visible.")]
    NotVisible,
    #[error("NavigationChild cannot be fetched because it contains an invalid route.")]
    InvalidRoute,
    #[error("NavigationChild cannot be fetched because it contains a route that misses a required parameter.")]
    MissingRequiredRouteParameter,
    #[error("NavigationChild cannot be fetched because it is unsupported.")]
    Unsupported,
}

impl From<sqlx::Error> for NavigationChildError {
    fn from(inner: sqlx::Error) -> Self {
        NavigationChildError::DatabaseError { inner }
    }
}

impl NavigationChild {
    pub fn fetch(
        id: i32,
        pool: &MySqlPool,
    ) -> Pin<Box<dyn Future<Output = Result<NavigationChild, NavigationChildError>> + '_>> {
        Box::pin(async move {
            let raw_navigation_child =
                RawNavigationChild::fetch(id, pool)
                    .await
                    .map_err(|error| match error {
                        RawNavigationChildError::DatabaseError { inner } => {
                            NavigationChildError::from(inner)
                        }
                    })?;

            if raw_navigation_child
                .parameters
                .get_or("visible", "true")
                .as_str()
                == "false"
            {
                return Err(NavigationChildError::NotVisible);
            }

            let mut children = Vec::with_capacity(raw_navigation_child.children.len());

            for child in raw_navigation_child.children.iter() {
                match NavigationChild::fetch(child.id, pool).await {
                    Ok(navigation_child) => children.push(navigation_child),
                    Err(error) => match error {
                        NavigationChildError::DatabaseError { inner } => return Err(inner.into()),
                        NavigationChildError::NotVisible => {}
                        NavigationChildError::InvalidRoute => {}
                        NavigationChildError::MissingRequiredRouteParameter => {}
                        NavigationChildError::Unsupported => {}
                    },
                }
            }

            let children = if children.is_empty() {
                None
            } else {
                Some(children)
            };

            if let (Some(label), Some(uri)) = (
                raw_navigation_child.parameters.get("label"),
                raw_navigation_child.parameters.get("uri"),
            ) {
                if uri.as_str() == "#" {
                    Ok(NavigationChild::Container(ContainerNavigationChild {
                        label,
                        children,
                    }))
                } else {
                    let re = Regex::new(r"^/(?P<id>\d+)$").unwrap();
                    match re.captures(&uri) {
                        Some(captures) => {
                            let id: i32 = captures.name("id").unwrap().as_str().parse().unwrap();
                            Ok(NavigationChild::Uuid(UuidNavigationChild {
                                label,
                                id,
                                children,
                            }))
                        }
                        _ => Ok(NavigationChild::Url(UrlNavigationChild {
                            label,
                            url: uri,
                            children,
                        })),
                    }
                }
            } else if let (Some(label), Some(route)) = (
                raw_navigation_child.parameters.get("label"),
                raw_navigation_child.parameters.get("route"),
            ) {
                match route.as_str() {
                    "blog" => Ok(NavigationChild::Url(UrlNavigationChild {
                        label,
                        url: "/blog".to_string(),
                        children,
                    })),
                    "discussion/discussions" => Ok(NavigationChild::Url(UrlNavigationChild {
                        label,
                        url: "/discussions".to_string(),
                        children,
                    })),
                    "discussion/discussions/get" => {
                        if let Some(id) = raw_navigation_child.parameters.get("params.id") {
                            Ok(NavigationChild::Url(UrlNavigationChild {
                                label,
                                url: format!("/discussions/{}", id),
                                children,
                            }))
                        } else {
                            Err(NavigationChildError::MissingRequiredRouteParameter)
                        }
                    }
                    "event/history/all" => Ok(NavigationChild::Url(UrlNavigationChild {
                        label,
                        url: "/event/history".to_string(),
                        children,
                    })),
                    "page/view" => {
                        if let Some(id) = raw_navigation_child.parameters.get("params.page") {
                            id.parse::<i32>()
                                .map_err(|_error| {
                                    NavigationChildError::MissingRequiredRouteParameter
                                })
                                .map(|id| {
                                    NavigationChild::Uuid(UuidNavigationChild {
                                        label,
                                        id,
                                        children,
                                    })
                                })
                        } else {
                            Err(NavigationChildError::MissingRequiredRouteParameter)
                        }
                    }
                    "subject/entity" => {
                        if let Some(subject) = raw_navigation_child.parameters.get("params.subject")
                        {
                            Ok(NavigationChild::Url(UrlNavigationChild {
                                label,
                                url: format!("/{}/entity/trash-bin", subject.replace(" ", "%20")),
                                children,
                            }))
                        } else {
                            Err(NavigationChildError::MissingRequiredRouteParameter)
                        }
                    }
                    "taxonomy/term/get" => {
                        if let Some(id) = raw_navigation_child.parameters.get("params.term") {
                            id.parse::<i32>()
                                .map_err(|_error| {
                                    NavigationChildError::MissingRequiredRouteParameter
                                })
                                .map(|id| {
                                    NavigationChild::Uuid(UuidNavigationChild {
                                        label: label.to_string(),
                                        id,
                                        children,
                                    })
                                })
                        } else {
                            Err(NavigationChildError::MissingRequiredRouteParameter)
                        }
                    }
                    "taxonomy/term/organize" => {
                        if let Some(id) = raw_navigation_child.parameters.get("params.term") {
                            Ok(NavigationChild::Url(UrlNavigationChild {
                                label,
                                url: format!("/taxonomy/term/organize/{}", id),
                                children,
                            }))
                        } else {
                            Err(NavigationChildError::MissingRequiredRouteParameter)
                        }
                    }
                    route => {
                        println!("Unhandled route: {}", route);
                        println!("Params: {:?}", raw_navigation_child.parameters);
                        Err(NavigationChildError::InvalidRoute)
                    }
                }
            } else {
                Err(NavigationChildError::Unsupported)
            }
        })
    }
}

struct RawNavigationChild {
    id: i32,
    children: Vec<RawNavigationChild>,
    parameters: RawNavigationChildParameters,
}

#[derive(Debug)]
pub struct RawNavigationChildParameters(HashMap<String, String>);

impl RawNavigationChildParameters {
    fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).map(|value| value.to_string())
    }

    fn get_or(&self, name: &str, default: &str) -> String {
        self.get(name).unwrap_or_else(|| default.to_string())
    }
}

#[derive(Error, Debug)]
pub enum RawNavigationChildError {
    #[error("RawNavigationChild cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

impl From<sqlx::Error> for RawNavigationChildError {
    fn from(inner: sqlx::Error) -> Self {
        RawNavigationChildError::DatabaseError { inner }
    }
}

impl RawNavigationChild {
    fn fetch(
        id: i32,
        pool: &MySqlPool,
    ) -> Pin<Box<dyn Future<Output = Result<RawNavigationChild, RawNavigationChildError>> + '_>>
    {
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
                children.push(RawNavigationChild::fetch(page.id, pool).await?);
            }

            let parameters = params
                .into_iter()
                .filter_map(|param| Some((param.name?, param.value?)))
                .collect();
            let parameters = RawNavigationChildParameters(parameters);

            Ok(RawNavigationChild {
                id,
                children,
                parameters,
            })
        })
    }
}
