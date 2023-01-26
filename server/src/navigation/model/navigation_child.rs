use std::collections::HashMap;
use std::convert::TryFrom;

use futures::try_join;
use regex::Regex;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

use crate::database::Executor;

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

impl TryFrom<(RawNavigationChild, Vec<NavigationChild>)> for NavigationChild {
    type Error = NavigationChildError;

    fn try_from(
        (raw_navigation_child, children): (RawNavigationChild, Vec<NavigationChild>),
    ) -> Result<Self, Self::Error> {
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
                            url: format!("/discussions/{id}"),
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
                            .map_err(|_error| NavigationChildError::MissingRequiredRouteParameter)
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
                    if let Some(subject) = raw_navigation_child.parameters.get("params.subject") {
                        Ok(NavigationChild::Url(UrlNavigationChild {
                            label,
                            url: format!("/{}/entity/trash-bin", subject.replace(' ', "%20")),
                            children,
                        }))
                    } else {
                        Err(NavigationChildError::MissingRequiredRouteParameter)
                    }
                }
                "taxonomy/term/get" => {
                    if let Some(id) = raw_navigation_child.parameters.get("params.term") {
                        id.parse::<i32>()
                            .map_err(|_error| NavigationChildError::MissingRequiredRouteParameter)
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
                            url: format!("/taxonomy/term/organize/{id}"),
                            children,
                        }))
                    } else {
                        Err(NavigationChildError::MissingRequiredRouteParameter)
                    }
                }
                route => {
                    println!("Unhandled route: {route}");
                    println!("Params: {:?}", raw_navigation_child.parameters);
                    Err(NavigationChildError::InvalidRoute)
                }
            }
        } else {
            Err(NavigationChildError::Unsupported)
        }
    }
}

pub struct RawNavigationChild {
    pub id: i32,
    pub children: Vec<i32>,
    pub parameters: RawNavigationChildParameters,
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
        Self::DatabaseError { inner }
    }
}

macro_rules! fetch_all_children {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT p.id
                    FROM navigation_page p
                    WHERE p.parent_id = ?
                    ORDER BY p.position, p.id
            "#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! fetch_all_parameters {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
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
            $id
        )
        .fetch_all($executor)
    }
}

macro_rules! to_raw_navigation_child {
    ($id: expr, $pages: expr, $params: expr) => {{
        let children = $pages.iter().map(|page| page.id).collect();
        let parameters = $params
            .into_iter()
            .filter_map(|param| Some((param.name?, param.value?)))
            .collect();
        let parameters = RawNavigationChildParameters(parameters);

        RawNavigationChild {
            id: $id,
            children,
            parameters,
        }
    }};
}

impl RawNavigationChild {
    pub async fn fetch(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<RawNavigationChild, RawNavigationChildError> {
        let pages = fetch_all_children!(id, pool);
        let params = fetch_all_parameters!(id, pool);

        let (pages, params) = try_join!(pages, params)?;

        let raw_navigation_child = to_raw_navigation_child!(id, pages, params);
        Ok(raw_navigation_child)
    }

    pub async fn fetch_via_transaction<'e, 'c, E>(
        id: i32,
        executor: E,
    ) -> Result<RawNavigationChild, RawNavigationChildError>
    where
        'c: 'e,
        E: 'c + Executor<'e>,
    {
        let mut transaction = executor.begin().await?;

        let pages = fetch_all_children!(id, &mut transaction).await?;
        let params = fetch_all_parameters!(id, &mut transaction).await?;

        let raw_navigation_child = to_raw_navigation_child!(id, pages, params);

        transaction.commit().await?;

        Ok(raw_navigation_child)
    }

    pub fn is_visible(&self) -> bool {
        self.parameters.get_or("visible", "true").as_str() != "false"
    }
}
