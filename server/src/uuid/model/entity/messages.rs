use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::Entity;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation, SuccessOutput};
use crate::uuid::abstract_entity_revision::EntityRevisionType;
use crate::uuid::{EntityType, Uuid};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EntityMessage {
    EntityAddRevisionMutation(entity_add_revision_mutation::Payload),
    EntityCheckoutRevisionMutation(checkout_revision_mutation::Payload),
    EntityCreateMutation(entity_create_mutation::Payload),
    EntityRejectRevisionMutation(reject_revision_mutation::Payload),
    UnrevisedEntitiesQuery(unrevised_entities_query::Payload),
    DeletedEntitiesQuery(deleted_entities_query::Payload),
    EntitySetLicenseMutation(entity_set_license_mutation::Payload),
    EntitySortMutation(entity_sort_mutation::Payload),
}

#[async_trait]
impl MessageResponder for EntityMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            EntityMessage::EntityAddRevisionMutation(message) => {
                message
                    .handle(acquire_from)
                    .await
            }
            EntityMessage::EntityCheckoutRevisionMutation(payload) => {
                payload
                    .handle(acquire_from)
                    .await
            }
            EntityMessage::EntityCreateMutation(message) => {
                message
                    .handle(acquire_from)
                    .await
            }
            EntityMessage::EntityRejectRevisionMutation(payload) => {
                payload
                    .handle(acquire_from)
                    .await
            }
            EntityMessage::UnrevisedEntitiesQuery(payload) => {
                payload
                    .handle(acquire_from)
                    .await
            }
            EntityMessage::DeletedEntitiesQuery(message) => {
                message
                    .handle(acquire_from)
                    .await
            }
            EntityMessage::EntitySetLicenseMutation(message) => {
                message
                    .handle(acquire_from)
                    .await
            }
            EntityMessage::EntitySortMutation(message) => {
                message
                    .handle(acquire_from)
                    .await
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

    #[derive(Debug, Deserialize, Serialize)]
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let entity_revision = Entity::add_revision(self, acquire_from).await?;
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

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub revision_id: i32,
        pub user_id: i32,
        pub reason: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Entity::checkout_revision(self, acquire_from).await?;
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

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub input: Input,
        pub entity_type: EntityType,
        pub user_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Uuid;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Entity::create(self, acquire_from).await?)
        }
    }
}

pub mod reject_revision_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub revision_id: i32,
        pub user_id: i32,
        pub reason: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Entity::reject_revision(self, acquire_from).await?;
            Ok(SuccessOutput { success: true })
        }
    }
}

pub mod unrevised_entities_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Output {
                unrevised_entity_ids: Entity::unrevised_entities(acquire_from).await?,
            })
        }
    }
}

pub mod deleted_entities_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let deleted_entities = Entity::deleted_entities(self, acquire_from).await?;
            Ok(Output {
                success: true,
                deleted_entities,
            })
        }
    }
}

pub mod entity_set_license_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Entity::set_license(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}

pub mod entity_sort_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub children_ids: Vec<i32>,
        pub entity_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Entity::sort(self, acquire_from).await?;
            Ok(SuccessOutput { success: true })
        }
    }
}
