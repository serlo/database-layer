use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    Entity, EntityAddRevisionError, EntityAddRevisionPayload, EntityCheckoutRevisionError,
    EntityCheckoutRevisionPayload, EntityRejectRevisionError, EntityRejectRevisionPayload,
};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};
use crate::uuid::abstract_entity_revision::EntityRevisionType;
use crate::uuid::{EntityAddRevisionInput, EntityType, Uuid};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EntityMessage {
    EntityAddRevisionMutation(EntityAddRevisionMutation),
    EntityCheckoutRevisionMutation(EntityCheckoutRevisionMutation),
    EntityCreateMutation(entity_create_mutation::Payload),
    EntityRejectRevisionMutation(EntityRejectRevisionMutation),
    UnrevisedEntitiesQuery(UnrevisedEntitiesQuery),
}

#[async_trait]
impl MessageResponder for EntityMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            EntityMessage::EntityAddRevisionMutation(message) => message.handle(connection).await,
            EntityMessage::EntityCheckoutRevisionMutation(message) => {
                message.handle(connection).await
            }
            EntityMessage::EntityCreateMutation(message) => {
                message.handle("EntityCreateMutation", connection).await
            }
            EntityMessage::EntityRejectRevisionMutation(message) => {
                message.handle(connection).await
            }
            EntityMessage::UnrevisedEntitiesQuery(message) => message.handle(connection).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityAddRevisionMutation {
    pub input: EntityAddRevisionInput,
    pub revision_type: EntityRevisionType,
    pub user_id: i32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRevisionData {
    pub success: bool,
    pub reason: Option<String>,
    pub revision_id: Option<i32>,
}

#[async_trait]
impl MessageResponder for EntityAddRevisionMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = EntityAddRevisionPayload {
            input: EntityAddRevisionInput {
                changes: self.input.changes.clone(),
                entity_id: self.input.entity_id,
                needs_review: self.input.needs_review,
                subscribe_this: self.input.subscribe_this,
                subscribe_this_by_email: self.input.subscribe_this_by_email,
                fields: self.input.fields.clone(),
            },
            revision_type: self.revision_type,
            user_id: self.user_id,
        };

        let entity_revision = match connection {
            Connection::Pool(pool) => Entity::add_revision(payload, pool).await,
            Connection::Transaction(transaction) => {
                Entity::add_revision(payload, transaction).await
            }
        };

        match entity_revision {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(AddRevisionData {
                    success: true,
                    reason: None,
                    revision_id: Some(data.id),
                }),
            Err(error) => {
                println!("/add-revision: {:?}", error);
                match error {
                    EntityAddRevisionError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityAddRevisionError::EventError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityAddRevisionError::UuidError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                    EntityAddRevisionError::EntityNotFound { .. } => HttpResponse::BadRequest()
                        .json(EntityRevisionData {
                            success: false,
                            reason: Some("no entity found for provided entityId".to_string()),
                        }),
                }
            }
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

pub mod entity_create_mutation {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Input {
        pub changes: String,
        pub instance: Instance,
        pub license_id: i32,
        pub subscribe_this: bool,
        pub needs_review: bool,
        pub subscribe_this_by_email: bool,
        pub fields: HashMap<String, String>,
        pub parent_id: Option<i32>,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub input: Input,
        pub entity_type: EntityType,
        pub user_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Uuid;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => Entity::create(self, pool).await.unwrap(),
                Connection::Transaction(transaction) => {
                    Entity::create(self, transaction).await.unwrap()
                }
            })
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
