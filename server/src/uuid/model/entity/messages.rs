use actix_web::HttpResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    Entity, EntityCheckoutRevisionError, EntityCheckoutRevisionPayload, EntityMetadata,
    EntityRejectRevisionError, EntityRejectRevisionPayload,
};
use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::Error;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EntityMessage {
    EntityCheckoutRevisionMutation(EntityCheckoutRevisionMutation),
    EntityRejectRevisionMutation(EntityRejectRevisionMutation),
    UnrevisedEntitiesQuery(UnrevisedEntitiesQuery),
    EntitiesMetadataQuery(entities_metadata_query::Payload),
}

#[async_trait]
impl MessageResponder for EntityMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            EntityMessage::EntityCheckoutRevisionMutation(message) => {
                message.handle(connection).await
            }
            EntityMessage::EntityRejectRevisionMutation(message) => {
                message.handle(connection).await
            }
            EntityMessage::UnrevisedEntitiesQuery(message) => message.handle(connection).await,
            EntityMessage::EntitiesMetadataQuery(payload) => {
                payload.handle("EntitiesQuery", connection).await
            }
        }
    }
}

pub mod entities_metadata_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub first: i32,
        pub after: Option<i32>,
        pub instance: Option<String>,
        pub modified_after: Option<DateTime<Utc>>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        entities: Vec<EntityMetadata>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            if self.first >= 10_000 {
                return Err(Error::BadRequest {
                    reason: "The 'first' value should be less than 10_000".to_string(),
                });
            };

            let entities = match connection {
                Connection::Pool(pool) => EntityMetadata::find_all(self, pool).await?,
                Connection::Transaction(transaction) => {
                    EntityMetadata::find_all(self, transaction).await?
                }
            };

            Ok(Output { entities })
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCheckoutRevisionMutation {
    pub revision_id: i32,
    pub user_id: i32,
    pub reason: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRevisionData {
    pub success: bool,
    pub reason: Option<String>,
}

#[async_trait]
impl MessageResponder for EntityCheckoutRevisionMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = EntityCheckoutRevisionPayload {
            revision_id: self.revision_id,
            user_id: self.user_id,
            reason: self.reason.to_string(),
        };
        let response = match connection {
            Connection::Pool(pool) => Entity::checkout_revision(payload, pool).await,
            Connection::Transaction(transaction) => {
                Entity::checkout_revision(payload, transaction).await
            }
        };
        match response {
            Ok(_) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(EntityRevisionData {
                    success: true,
                    reason: None,
                }),
            Err(e) => {
                println!("/checkout-revision: {:?}", e);
                match e {
                    EntityCheckoutRevisionError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityCheckoutRevisionError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityCheckoutRevisionError::UuidError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityCheckoutRevisionError::RevisionAlreadyCheckedOut => {
                        HttpResponse::BadRequest().json(EntityRevisionData {
                            success: false,
                            reason: Some("revision is already checked out".to_string()),
                        })
                    }
                    EntityCheckoutRevisionError::InvalidRevision { .. } => {
                        HttpResponse::BadRequest().json(EntityRevisionData {
                            success: false,
                            reason: Some("revision invalid".to_string()),
                        })
                    }
                    EntityCheckoutRevisionError::InvalidRepository { .. } => {
                        HttpResponse::BadRequest().json(EntityRevisionData {
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
pub struct EntityRejectRevisionMutation {
    pub revision_id: i32,
    pub user_id: i32,
    pub reason: String,
}

#[async_trait]
impl MessageResponder for EntityRejectRevisionMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = EntityRejectRevisionPayload {
            revision_id: self.revision_id,
            user_id: self.user_id,
            reason: self.reason.to_string(),
        };
        let response = match connection {
            Connection::Pool(pool) => Entity::reject_revision(payload, pool).await,
            Connection::Transaction(transaction) => {
                Entity::reject_revision(payload, transaction).await
            }
        };
        match response {
            Ok(_) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(EntityRevisionData {
                    success: true,
                    reason: None,
                }),
            Err(e) => {
                println!("/reject-revision: {:?}", e);
                match e {
                    EntityRejectRevisionError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityRejectRevisionError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityRejectRevisionError::UuidError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityRejectRevisionError::RevisionAlreadyRejected => {
                        HttpResponse::BadRequest().json(EntityRevisionData {
                            success: false,
                            reason: Some("revision has already been rejected".to_string()),
                        })
                    }
                    EntityRejectRevisionError::RevisionCurrentlyCheckedOut => {
                        HttpResponse::BadRequest().json(EntityRevisionData {
                            success: false,
                            reason: Some("revision is checked out currently".to_string()),
                        })
                    }
                    EntityRejectRevisionError::InvalidRevision { .. } => HttpResponse::BadRequest()
                        .json(EntityRevisionData {
                            success: false,
                            reason: Some("revision invalid".to_string()),
                        }),
                    EntityRejectRevisionError::InvalidRepository { .. } => {
                        HttpResponse::BadRequest().json(EntityRevisionData {
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
pub struct UnrevisedEntitiesQuery {}

#[async_trait]
impl MessageResponder for UnrevisedEntitiesQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let response = match connection {
            Connection::Pool(pool) => Entity::unrevised_entities(pool).await,
            Connection::Transaction(transaction) => Entity::unrevised_entities(transaction).await,
        };
        match response {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/unrevised-entities: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
