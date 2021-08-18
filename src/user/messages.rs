use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::User;
use crate::database::Connection;
use crate::message::{MessageResponder, Operation, OperationError};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UserMessage {
    ActiveAuthorsQuery(Option<serde_json::Value>),
    ActiveReviewersQuery(Option<serde_json::Value>),
    ActivityByTypeQuery(activity_by_type_query::Payload),
}

#[async_trait]
impl MessageResponder for UserMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            UserMessage::ActiveAuthorsQuery(_) => active_authors_query(connection).await,
            UserMessage::ActiveReviewersQuery(_) => active_reviewers_query(connection).await,
            UserMessage::ActivityByTypeQuery(payload) => {
                payload.handle("ActivityByTypeQuery", connection).await
            }
        }
    }
}

async fn active_authors_query(connection: Connection<'_, '_>) -> HttpResponse {
    let active_authors = match connection {
        Connection::Pool(pool) => User::fetch_active_authors(pool).await,
        Connection::Transaction(transaction) => User::fetch_active_authors(transaction).await,
    };
    match active_authors {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/user/active-authors: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn active_reviewers_query(connection: Connection<'_, '_>) -> HttpResponse {
    let active_reviewers = match connection {
        Connection::Pool(pool) => User::fetch_active_reviewers(pool).await,
        Connection::Transaction(transaction) => User::fetch_active_reviewers(transaction).await,
    };
    match active_reviewers {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/user/active-reviewers: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub mod activity_by_type_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
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

        async fn execute(&self, connection: Connection<'_, '_>) -> Result<Output, OperationError> {
            Ok(match connection {
                Connection::Pool(pool) => User::fetch_activity_by_type(self.user_id, pool).await?,
                Connection::Transaction(transaction) => {
                    User::fetch_activity_by_type(self.user_id, transaction).await?
                }
            })
        }
    }
}
