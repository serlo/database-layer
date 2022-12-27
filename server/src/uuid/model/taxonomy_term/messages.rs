use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::TaxonomyTerm;
use crate::database::Connection;
use crate::instance::Instance;
use crate::message::MessageResponder;
use crate::operation::{self, Operation, SuccessOutput};
use crate::uuid::{TaxonomyType, Uuid};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum TaxonomyTermMessage {
    DeletedTaxonomiesQuery(deleted_taxonomies_query::Payload),
    TaxonomyTermSetNameAndDescriptionMutation(
        taxonomy_term_set_name_and_description_mutation::Payload,
    ),
    TaxonomyTermCreateMutation(taxonomy_term_create_mutation::Payload),
    TaxonomyCreateEntityLinksMutation(taxonomy_create_entity_links_mutation::Payload),
    TaxonomyDeleteEntityLinksMutation(taxonomy_delete_entity_links_mutation::Payload),
    TaxonomySortMutation(taxonomy_sort_mutation::Payload),
}

#[async_trait]
impl MessageResponder for TaxonomyTermMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            TaxonomyTermMessage::DeletedTaxonomiesQuery(payload) => {
                payload.handle("DeletedTaxonomiesQuery", connection).await
            }
            TaxonomyTermMessage::TaxonomyTermSetNameAndDescriptionMutation(payload) => {
                payload
                    .handle("TaxonomyTermSetNameAndDescriptionMutation", connection)
                    .await
            }
            TaxonomyTermMessage::TaxonomyTermCreateMutation(payload) => {
                payload
                    .handle("TaxonomyTermCreateMutation", connection)
                    .await
            }
            TaxonomyTermMessage::TaxonomyCreateEntityLinksMutation(payload) => {
                payload
                    .handle("TaxonomyCreateEntityLinksMutation", connection)
                    .await
            }
            TaxonomyTermMessage::TaxonomyDeleteEntityLinksMutation(payload) => {
                payload
                    .handle("TaxonomyDeleteEntityLinksMutation", connection)
                    .await
            }
            TaxonomyTermMessage::TaxonomySortMutation(payload) => {
                payload.handle("TaxonomySortMutation", connection).await
            }
        }
    }
}

// It is an almost identical code to deleted_entities_query. See if we want to abstract it
pub mod deleted_taxonomies_query {
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
    pub struct DeletedTaxonomyTerm {
        pub id: i32,
        pub date_of_deletion: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        success: bool,
        deleted_taxonomies: Vec<DeletedTaxonomyTerm>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let deleted_taxonomies = match connection {
                Connection::Pool(pool) => TaxonomyTerm::deleted_taxonomies(self, pool).await?,
                Connection::Transaction(transaction) => {
                    TaxonomyTerm::deleted_taxonomies(self, transaction).await?
                }
            };
            Ok(Output {
                success: true,
                deleted_taxonomies,
            })
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

pub mod taxonomy_create_entity_links_mutation {
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

pub mod taxonomy_sort_mutation {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub children_ids: Vec<i32>,
        pub taxonomy_term_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => TaxonomyTerm::sort(self, pool).await?,
                Connection::Transaction(transaction) => {
                    TaxonomyTerm::sort(self, transaction).await?
                }
            }
            Ok(SuccessOutput { success: true })
        }
    }
}
