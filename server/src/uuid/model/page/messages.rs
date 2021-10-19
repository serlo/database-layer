use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::database::Connection;
use crate::message::MessageResponder;

use super::{
    Page, PageCheckoutRevisionError, PageCheckoutRevisionPayload, PageRejectRevisionError,
    PageRejectRevisionPayload,
};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum PageMessage {
    PageCheckoutRevisionMutation(PageCheckoutRevisionMutation),
    PageRejectRevisionMutation(PageRejectRevisionMutation),
}

#[async_trait]
impl MessageResponder for PageMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            PageMessage::PageCheckoutRevisionMutation(message) => message.handle(connection).await,
            PageMessage::PageRejectRevisionMutation(message) => message.handle(connection).await,
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
