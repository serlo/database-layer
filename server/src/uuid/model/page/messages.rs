use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};
use crate::uuid::{PageAddRevisionError, PageCreateError, PageCreatePayload};

use super::{
    Page, PageCheckoutRevisionError, PageCheckoutRevisionPayload, PageRejectRevisionError,
    PageRejectRevisionPayload,
};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum PageMessage {
    PageAddRevisionMutation(add_revision_mutation::Payload),
    PageCheckoutRevisionMutation(PageCheckoutRevisionMutation),
    PageCreateMutation(PageCreateMutation),
    PageRejectRevisionMutation(PageRejectRevisionMutation),
}

#[async_trait]
impl MessageResponder for PageMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            PageMessage::PageAddRevisionMutation(payload) => {
                payload.handle("PageAddRevisionMutation", connection).await
            }
            PageMessage::PageCheckoutRevisionMutation(message) => message.handle(connection).await,
            PageMessage::PageCreateMutation(message) => message.handle(connection).await,
            PageMessage::PageRejectRevisionMutation(message) => message.handle(connection).await,
        }
    }
}

pub mod add_revision_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub content: String,
        pub title: String,
        pub page_id: i32,
        pub user_id: i32,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub revision_id: i32,
        pub success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let page_revision = match connection {
                Connection::Pool(pool) => Page::add_revision(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Page::add_revision(self, transaction).await?
                }
            };
            Ok(Output {
                revision_id: page_revision.id,
                success: true,
            })
        }
    }

    impl From<PageAddRevisionError> for operation::Error {
        fn from(error: PageAddRevisionError) -> Self {
            match error {
                PageAddRevisionError::PageNotFound { .. } => operation::Error::BadRequest {
                    reason: "no page found for provided pageId".to_string(),
                },
                _ => operation::Error::InternalServerError {
                    error: Box::new(error),
                },
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageCheckoutRevisionMutation {
    pub revision_id: i32,
    pub user_id: i32,
    pub reason: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRevisionData {
    pub success: bool,
    pub reason: Option<String>,
}

#[async_trait]
impl MessageResponder for PageCheckoutRevisionMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = PageCheckoutRevisionPayload {
            revision_id: self.revision_id,
            user_id: self.user_id,
            reason: self.reason.to_string(),
        };
        let response = match connection {
            Connection::Pool(pool) => Page::checkout_revision(payload, pool).await,
            Connection::Transaction(transaction) => {
                Page::checkout_revision(payload, transaction).await
            }
        };
        match response {
            Ok(_) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(PageRevisionData {
                    success: true,
                    reason: None,
                }),
            Err(e) => {
                println!("/checkout-revision: {:?}", e);
                match e {
                    PageCheckoutRevisionError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageCheckoutRevisionError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageCheckoutRevisionError::UuidError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageCheckoutRevisionError::RevisionAlreadyCheckedOut => {
                        HttpResponse::BadRequest().json(PageRevisionData {
                            success: false,
                            reason: Some("revision is already checked out".to_string()),
                        })
                    }
                    PageCheckoutRevisionError::InvalidRevision { .. } => HttpResponse::BadRequest()
                        .json(PageRevisionData {
                            success: false,
                            reason: Some("revision invalid".to_string()),
                        }),
                    PageCheckoutRevisionError::InvalidRepository { .. } => {
                        HttpResponse::BadRequest().json(PageRevisionData {
                            success: false,
                            reason: Some("repository invalid".to_string()),
                        })
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageCreateMutation {
    pub content: String,
    pub discussions_enabled: bool,
    pub forum_id: Option<i32>,
    pub instance: Instance,
    pub license_id: i32,
    pub title: String,
    pub user_id: i32,
}

#[async_trait]
impl MessageResponder for PageCreateMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = PageCreatePayload {
            content: self.content.clone(),
            discussions_enabled: self.discussions_enabled,
            forum_id: self.forum_id,
            instance: self.instance.clone(),
            license_id: self.license_id,
            title: self.title.clone(),
            user_id: self.user_id,
        };
        let response = match connection {
            Connection::Pool(pool) => Page::create(payload, pool).await,
            Connection::Transaction(transaction) => Page::create(payload, transaction).await,
        };
        match response {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/create-page: {:?}", e);
                match e {
                    PageCreateError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageCreateError::RevisionError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageCreateError::UuidError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRejectRevisionMutation {
    pub revision_id: i32,
    pub user_id: i32,
    pub reason: String,
}

#[async_trait]
impl MessageResponder for PageRejectRevisionMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = PageRejectRevisionPayload {
            revision_id: self.revision_id,
            user_id: self.user_id,
            reason: self.reason.to_string(),
        };
        let response = match connection {
            Connection::Pool(pool) => Page::reject_revision(payload, pool).await,
            Connection::Transaction(transaction) => {
                Page::reject_revision(payload, transaction).await
            }
        };
        match response {
            Ok(_) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(PageRevisionData {
                    success: true,
                    reason: None,
                }),
            Err(e) => {
                println!("/reject-revision: {:?}", e);
                match e {
                    PageRejectRevisionError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageRejectRevisionError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageRejectRevisionError::UuidError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    PageRejectRevisionError::RevisionAlreadyRejected => HttpResponse::BadRequest()
                        .json(PageRevisionData {
                            success: false,
                            reason: Some("revision has already been rejected".to_string()),
                        }),
                    PageRejectRevisionError::RevisionCurrentlyCheckedOut => {
                        HttpResponse::BadRequest().json(PageRevisionData {
                            success: false,
                            reason: Some("revision is checked out currently".to_string()),
                        })
                    }
                    PageRejectRevisionError::InvalidRevision { .. } => HttpResponse::BadRequest()
                        .json(PageRevisionData {
                            success: false,
                            reason: Some("revision invalid".to_string()),
                        }),
                    PageRejectRevisionError::InvalidRepository { .. } => HttpResponse::BadRequest()
                        .json(PageRevisionData {
                            success: false,
                            reason: Some("repository invalid".to_string()),
                        }),
                }
            }
        }
    }
}
