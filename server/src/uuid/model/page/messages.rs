use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};
use crate::uuid::Uuid;

use super::{Page, PageCheckoutRevisionError, PageRejectRevisionError};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum PageMessage {
    PageAddRevisionMutation(add_revision_mutation::Payload),
    PageCheckoutRevisionMutation(checkout_revision_mutation::Payload),
    PageCreateMutation(create_mutation::Payload),
    PageRejectRevisionMutation(reject_revision_mutation::Payload),
    PagesQuery(pages_query::Payload),
}

#[async_trait]
impl MessageResponder for PageMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            PageMessage::PageAddRevisionMutation(payload) => {
                payload
                    .handle("PageAddRevisionMutation", acquire_from)
                    .await
            }
            PageMessage::PageCheckoutRevisionMutation(payload) => {
                payload
                    .handle("PageCheckoutRevisionMutation", acquire_from)
                    .await
            }
            PageMessage::PageCreateMutation(payload) => {
                payload.handle("PageCreateMutation", acquire_from).await
            }
            PageMessage::PageRejectRevisionMutation(payload) => {
                payload
                    .handle("PageRejectRevisionMutation", acquire_from)
                    .await
            }
            PageMessage::PagesQuery(payload) => payload.handle("PagesQuery", acquire_from).await,
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let page_revision = Page::add_revision(self, acquire_from).await?;
            Ok(Output {
                revision_id: page_revision.id,
                success: true,
            })
        }
    }
}

pub mod checkout_revision_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub revision_id: i32,
        pub user_id: i32,
        pub reason: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = operation::SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Page::checkout_revision(self, acquire_from).await?;
            Ok(operation::SuccessOutput { success: true })
        }
    }
    impl From<PageCheckoutRevisionError> for operation::Error {
        fn from(e: PageCheckoutRevisionError) -> Self {
            match e {
                PageCheckoutRevisionError::DatabaseError { .. }
                | PageCheckoutRevisionError::EventError { .. }
                | PageCheckoutRevisionError::UuidError { .. } => {
                    operation::Error::InternalServerError { error: Box::new(e) }
                }
                PageCheckoutRevisionError::RevisionAlreadyCheckedOut => {
                    operation::Error::BadRequest {
                        reason: "revision is already checked out".to_string(),
                    }
                }
                PageCheckoutRevisionError::InvalidRevision { .. } => operation::Error::BadRequest {
                    reason: "revision invalid".to_string(),
                },
                PageCheckoutRevisionError::InvalidRepository { .. } => {
                    operation::Error::BadRequest {
                        reason: "repository invalid".to_string(),
                    }
                }
            }
        }
    }
}

pub mod create_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub content: String,
        pub discussions_enabled: bool,
        pub forum_id: Option<i32>,
        pub instance: Instance,
        pub license_id: i32,
        pub title: String,
        pub user_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Uuid;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Page::create(self, acquire_from).await?)
        }
    }
}

pub mod reject_revision_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub revision_id: i32,
        pub user_id: i32,
        pub reason: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = operation::SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Page::reject_revision(self, acquire_from).await?;
            Ok(operation::SuccessOutput { success: true })
        }
    }
    impl From<PageRejectRevisionError> for operation::Error {
        fn from(e: PageRejectRevisionError) -> Self {
            match e {
                PageRejectRevisionError::DatabaseError { .. }
                | PageRejectRevisionError::EventError { .. }
                | PageRejectRevisionError::UuidError { .. } => {
                    operation::Error::InternalServerError { error: Box::new(e) }
                }
                PageRejectRevisionError::RevisionAlreadyRejected => operation::Error::BadRequest {
                    reason: "revision has already been rejected".to_string(),
                },
                PageRejectRevisionError::RevisionCurrentlyCheckedOut => {
                    operation::Error::BadRequest {
                        reason: "revision is checked out currently".to_string(),
                    }
                }
                PageRejectRevisionError::InvalidRevision { .. } => operation::Error::BadRequest {
                    reason: "revision invalid".to_string(),
                },
                PageRejectRevisionError::InvalidRepository { .. } => operation::Error::BadRequest {
                    reason: "repository invalid".to_string(),
                },
            }
        }
    }
}

pub mod pages_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub instance: Option<Instance>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub pages: Vec<i32>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let result = Page::fetch_all_pages(self, acquire_from).await?;
            Ok(Output { pages: result })
        }
    }
}
