use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::TaxonomyTerm;
use crate::message::MessageResponder;
use crate::operation::{self, Operation, SuccessOutput};
use crate::uuid::{TaxonomyType, Uuid};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum TaxonomyTermMessage {
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
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        self.handle(acquire_from).await
    }
}

pub mod taxonomy_term_set_name_and_description_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            TaxonomyTerm::set_name_and_description(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}

pub mod taxonomy_term_create_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(TaxonomyTerm::create(self, acquire_from).await?)
        }
    }
}

pub mod taxonomy_create_entity_links_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub entity_ids: Vec<i32>,
        pub taxonomy_term_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            TaxonomyTerm::create_entity_link(self, acquire_from).await?;
            Ok(SuccessOutput { success: true })
        }
    }
}

pub mod taxonomy_delete_entity_links_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub entity_ids: Vec<i32>,
        pub taxonomy_term_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            TaxonomyTerm::delete_entity_link(self, acquire_from).await?;
            Ok(SuccessOutput { success: true })
        }
    }
}

pub mod taxonomy_sort_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub children_ids: Vec<i32>,
        pub taxonomy_term_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            TaxonomyTerm::sort(self, acquire_from).await?;
            Ok(SuccessOutput { success: true })
        }
    }
}
