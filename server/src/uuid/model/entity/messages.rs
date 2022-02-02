use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    Entity, EntityAddRevisionError, EntityAddRevisionPayload, EntityCheckoutRevisionError,
    EntityCheckoutRevisionPayload, EntityRejectRevisionError, EntityRejectRevisionPayload,
};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EntityMessage {
    EntityAddRevisionMutation(EntityAddRevisionMutation),
    EntityCheckoutRevisionMutation(EntityCheckoutRevisionMutation),
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
    pub changes: String,
    pub content: String,
    pub needs_review: bool,
    pub subscribe_this_by_email: bool,
    pub subscribe_this: bool,
    pub entity_id: i32,
    pub meta_description: Option<String>,
    pub meta_title: Option<String>,
    pub title: String,
    pub user_id: i32,
}

#[async_trait]
impl MessageResponder for EntityAddRevisionMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = EntityAddRevisionPayload {
            changes: self.changes.clone(),
            content: self.content.clone(),
            needs_review: self.needs_review,
            subscribe_this_by_email: self.subscribe_this_by_email,
            subscribe_this: self.subscribe_this,
            entity_id: self.entity_id,
            meta_description: self.meta_description.clone(),
            meta_title: self.meta_title.clone(),
            title: self.title.clone(),
            user_id: self.user_id,
        };

        let entity_revision = match connection {
            Connection::Pool(pool) => Entity::add_revision(payload, pool).await,
            Connection::Transaction(transaction) => {
                Entity::add_revision(payload, transaction).await
            }
        };

        match entity_revision {
            Ok(_) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(EntityRevisionData {
                    success: true,
                    reason: None,
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
                    EntityAddRevisionError::InvalidRevision { .. } => HttpResponse::BadRequest()
                        .json(EntityRevisionData {
                            success: false,
                            reason: Some("revision invalid".to_string()),
                        }),
                    EntityAddRevisionError::InvalidRepository { .. } => HttpResponse::BadRequest()
                        .json(EntityRevisionData {
                            success: false,
                            reason: Some("repository invalid".to_string()),
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
