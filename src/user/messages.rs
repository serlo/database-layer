use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::User;
use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UserMessage {
    ActiveAuthorsQuery(Option<serde_json::Value>),
    ActiveReviewersQuery(Option<serde_json::Value>),
    // TODO: Delete when not needed any more in api.serlo.org
    // See https://github.com/serlo/api.serlo.org/issues/459
    ActivityByTypeQuery(user_activity_by_type_query::Payload),
    UserActivityByTypeQuery(user_activity_by_type_query::Payload),
    UserDeleteBotsMutation(user_delete_bots_mutation::Payload),
    // UserDeleteUsersMutation(user_delete_users_mutation::Payload),
}

#[async_trait]
impl MessageResponder for UserMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            UserMessage::ActiveAuthorsQuery(_) => {
                active_authors_query::Payload {}
                    .handle("ActiveAuthorsQuery", connection)
                    .await
            }
            UserMessage::ActiveReviewersQuery(_) => {
                active_reviewers_query::Payload {}
                    .handle("ActiveReviewersQuery", connection)
                    .await
            }
            UserMessage::ActivityByTypeQuery(payload) => {
                payload.handle("ActivityByTypeQuery", connection).await
            }
            UserMessage::UserActivityByTypeQuery(payload) => {
                payload.handle("ActivityByTypeQuery", connection).await
            }
            UserMessage::UserDeleteBotsMutation(payload) => {
                payload.handle("UserDeleteBotsMutation", connection).await
            }
        }
    }
}

pub mod active_authors_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {}

    #[async_trait]
    impl Operation for Payload {
        type Output = Vec<i32>;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => User::fetch_active_authors(pool).await?,
                Connection::Transaction(transaction) => {
                    User::fetch_active_authors(transaction).await?
                }
            })
        }
    }
}

pub mod active_reviewers_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {}

    #[async_trait]
    impl Operation for Payload {
        type Output = Vec<i32>;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => User::fetch_active_reviewers(pool).await?,
                Connection::Transaction(transaction) => {
                    User::fetch_active_reviewers(transaction).await?
                }
            })
        }
    }
}

pub mod user_activity_by_type_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        user_id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub edits: i32,
        pub reviews: i32,
        pub comments: i32,
        pub taxonomy: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => User::fetch_activity_by_type(self.user_id, pool).await?,
                Connection::Transaction(transaction) => {
                    User::fetch_activity_by_type(self.user_id, transaction).await?
                }
            })
        }
    }
}

pub mod user_delete_bots_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_ids: Vec<i32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => User::delete_bot(self, pool).await?,
                Connection::Transaction(transaction) => User::delete_bot(self, transaction).await?,
            };
            Ok(Output { success: true })
        }
    }
}
