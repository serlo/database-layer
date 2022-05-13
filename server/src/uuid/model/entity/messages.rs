use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{Entity, EntityRejectRevisionError, EntityRejectRevisionPayload};
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation, SuccessOutput};
use crate::uuid::abstract_entity_revision::EntityRevisionType;
use crate::uuid::{EntityType, Uuid};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EntityMessage {
    EntityAddRevisionMutation(entity_add_revision_mutation::Payload),
    EntityCheckoutRevisionMutation(checkout_revision_mutation::Payload),
    EntityCreateMutation(entity_create_mutation::Payload),
    EntityRejectRevisionMutation(reject_revision_mutation::Payload),
    UnrevisedEntitiesQuery(UnrevisedEntitiesQuery),
    DeletedEntitiesQuery(deleted_entities_query::Payload),
}

#[async_trait]
impl MessageResponder for EntityMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            EntityMessage::EntityAddRevisionMutation(message) => {
                message
                    .handle("EntityAddRevisionMutation", connection)
                    .await
            }
            EntityMessage::EntityCheckoutRevisionMutation(payload) => {
                payload
                    .handle("EntityCheckoutRevisionMutation", connection)
                    .await
            }
            EntityMessage::EntityCreateMutation(message) => {
                message.handle("EntityCreateMutation", connection).await
            }
            EntityMessage::EntityRejectRevisionMutation(payload) => {
                payload
                    .handle("EntityRejectRevisionMutation", connection)
                    .await
            }
            EntityMessage::UnrevisedEntitiesQuery(message) => message.handle(connection).await,
            EntityMessage::DeletedEntitiesQuery(message) => {
                message.handle("DeletedEntitiesQuery", connection).await
            }
        }
    }
}

pub mod entity_add_revision_mutation {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Input {
        pub changes: String,
        pub entity_id: i32,
        pub needs_review: bool,
        pub subscribe_this: bool,
        pub subscribe_this_by_email: bool,
        pub fields: HashMap<String, String>,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub input: Input,
        pub revision_type: EntityRevisionType,
        pub user_id: i32,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
        pub reason: Option<String>,
        pub revision_id: Option<i32>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let entity_revision = match connection {
                Connection::Pool(pool) => Entity::add_revision(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Entity::add_revision(self, transaction).await?
                }
            };

            Ok(Output {
                success: true,
                reason: None,
                revision_id: Some(entity_revision.id),
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
        type Output = SuccessOutput;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => Entity::checkout_revision(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Entity::checkout_revision(self, transaction).await?
                }
            };
            Ok(SuccessOutput { success: true })
        }
    }
}

pub mod entity_create_mutation {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Input {
        pub changes: String,
        pub license_id: i32,
        pub subscribe_this: bool,
        pub needs_review: bool,
        pub subscribe_this_by_email: bool,
        pub fields: HashMap<String, String>,
        pub parent_id: Option<i32>,
        pub taxonomy_term_id: Option<i32>,
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
                Connection::Pool(pool) => Entity::create(self, pool).await?,
                Connection::Transaction(transaction) => Entity::create(self, transaction).await?,
            })
        }
    }
}

pub mod reject_revision_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EntityRevisionData {
        pub success: bool,
        pub reason: Option<String>,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub revision_id: i32,
        pub user_id: i32,
        pub reason: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = EntityRevisionData;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let payload = EntityRejectRevisionPayload {
                revision_id: self.revision_id,
                user_id: self.user_id,
                reason: self.reason.to_string(),
            };
            match connection {
                Connection::Pool(pool) => Entity::reject_revision(payload, pool).await?,
                Connection::Transaction(transaction) => {
                    Entity::reject_revision(payload, transaction).await?
                }
            };
            Ok(EntityRevisionData {
                success: true,
                reason: None,
            })
        }
    }
    impl From<EntityRejectRevisionError> for operation::Error {
        fn from(e: EntityRejectRevisionError) -> Self {
            match e {
                EntityRejectRevisionError::DatabaseError { .. }
                | EntityRejectRevisionError::EventError { .. }
                | EntityRejectRevisionError::UuidError { .. } => {
                    operation::Error::InternalServerError { error: Box::new(e) }
                }
                EntityRejectRevisionError::RevisionAlreadyRejected => {
                    operation::Error::BadRequest {
                        reason: "revision has already been rejected".to_string(),
                    }
                }
                EntityRejectRevisionError::RevisionCurrentlyCheckedOut => {
                    operation::Error::BadRequest {
                        reason: "revision is checked out currently".to_string(),
                    }
                }
                EntityRejectRevisionError::InvalidRevision { .. } => operation::Error::BadRequest {
                    reason: "revision invalid".to_string(),
                },
                EntityRejectRevisionError::InvalidRepository { .. } => {
                    operation::Error::BadRequest {
                        reason: "repository invalid".to_string(),
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

pub mod deleted_entities_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub first: i32,
        pub after: Option<String>,
        pub instance: Option<Instance>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DeletedEntity {
        pub date_of_deletion: String,
        pub id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        success: bool,
        deleted_entities: Vec<DeletedEntity>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let deleted_entities = match connection {
                Connection::Pool(pool) => Entity::deleted_entities(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Entity::deleted_entities(self, transaction).await?
                }
            };
            Ok(Output {
                success: true,
                deleted_entities,
            })
        }
    }
}
