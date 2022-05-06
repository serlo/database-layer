use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::TaxonomyTerm;
use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};
use crate::uuid::{TaxonomyType, Uuid};
use crate::SuccessOutput;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum TaxonomyTermMessage {
    TaxonomyTermSetNameAndDescriptionMutation(
        taxonomy_term_set_name_and_description_mutation::Payload,
    ),
    TaxonomyTermMoveMutation(taxonomy_term_move_mutation::Payload),
    TaxonomyTermCreateMutation(taxonomy_term_create_mutation::Payload),
    TaxonomyCreateEntityLinkMutation(taxonomy_create_entity_link_mutation::Payload),
    TaxonomyDeleteEntityLinksMutation(taxonomy_delete_entity_links_mutation::Payload),
}

#[async_trait]
impl MessageResponder for TaxonomyTermMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            TaxonomyTermMessage::TaxonomyTermSetNameAndDescriptionMutation(payload) => {
                payload
                    .handle("TaxonomyTermSetNameAndDescriptionMutation", connection)
                    .await
            }
            TaxonomyTermMessage::TaxonomyTermMoveMutation(payload) => {
                payload.handle("TaxonomyTermMoveMutation", connection).await
            }
            TaxonomyTermMessage::TaxonomyTermCreateMutation(payload) => {
                payload
                    .handle("TaxonomyTermCreateMutation", connection)
                    .await
            }
            TaxonomyTermMessage::TaxonomyCreateEntityLinkMutation(payload) => {
                payload
                    .handle("TaxonomyCreateEntityLinkMutation", connection)
                    .await
            }
            TaxonomyTermMessage::TaxonomyDeleteEntityLinksMutation(payload) => {
                payload
                    .handle("TaxonomyDeleteEntityLinksMutation", connection)
                    .await
            }
        }
    }
}

pub mod taxonomy_term_set_name_and_description_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub id: i32,
        pub user_id: i32,
        pub name: String,
        pub description: Option<String>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => {
                    TaxonomyTerm::set_name_and_description(self, pool).await?
                }
                Connection::Transaction(transaction) => {
                    TaxonomyTerm::set_name_and_description(self, transaction).await?
                }
            };

            Ok(Output { success: true })
        }
    }
}

pub mod taxonomy_term_move_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub children_ids: Vec<i32>,
        pub destination: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => TaxonomyTerm::move_to_new_parent(self, pool).await?,
                Connection::Transaction(transaction) => {
                    TaxonomyTerm::move_to_new_parent(self, transaction).await?
                }
            };

            Ok(SuccessOutput { success: true })
        }
    }
}

pub mod taxonomy_term_create_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub taxonomy_type: TaxonomyType,
        pub parent_id: i32,
        pub name: String,
        pub description: Option<String>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Uuid;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => TaxonomyTerm::create(self, pool).await?,
                Connection::Transaction(transaction) => {
                    TaxonomyTerm::create(self, transaction).await?
                }
            })
        }
    }
}

pub mod taxonomy_create_entity_link_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub entity_ids: Vec<i32>,
        pub taxonomy_term_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => TaxonomyTerm::create_entity_link(self, pool).await?,
                Connection::Transaction(transaction) => {
                    TaxonomyTerm::create_entity_link(self, transaction).await?
                }
            }
            Ok(SuccessOutput { success: true })
        }
    }
}

pub mod taxonomy_delete_entity_links_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub entity_ids: Vec<i32>,
        pub taxonomy_term_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => TaxonomyTerm::delete_entity_link(self, pool).await?,
                Connection::Transaction(transaction) => {
                    TaxonomyTerm::delete_entity_link(self, transaction).await?
                }
            }
            Ok(SuccessOutput { success: true })
        }
    }
}
