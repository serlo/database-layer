use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::Entity;
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
    UnrevisedEntitiesQuery(unrevised_entities_query::Payload),
    DeletedEntitiesQuery(deleted_entities_query::Payload),
    EntitySetLicenseMutation(entity_set_license_mutation::Payload),
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
            EntityMessage::UnrevisedEntitiesQuery(payload) => {
                payload.handle("UnrevisedEntitiesQuery", connection).await
            }
            EntityMessage::DeletedEntitiesQuery(message) => {
                message.handle("DeletedEntitiesQuery", connection).await
            }
            EntityMessage::EntitySetLicenseMutation(message) => {
                message.handle("EntitySetLicenseMutation", connection).await
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
                Connection::Pool(pool) => Entity::reject_revision(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Entity::reject_revision(self, transaction).await?
                }
            };
            Ok(SuccessOutput { success: true })
        }
    }
}

pub mod unrevised_entities_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {}

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub unrevised_entity_ids: Vec<i32>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(Output {
                unrevised_entity_ids: match connection {
                    Connection::Pool(pool) => Entity::unrevised_entities(pool).await?,
                    Connection::Transaction(transaction) => {
                        Entity::unrevised_entities(transaction).await?
                    }
                },
            })
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

pub mod entity_set_license_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub entity_id: i32,
        pub license_id: i32,
        pub user_id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => Entity::set_license(self, pool).await?,
                Connection::Transaction(transaction) => {
                    Entity::set_license(self, transaction).await?
                }
            };
            Ok(Output { success: true })
        }
    }
}
