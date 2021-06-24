use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{User, UserError};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum UserQueriesMessage {
    ActiveAuthorsQuery(ActiveAuthorsQuery),
    ActiveReviewersQuery(ActiveReviewersQuery),
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UserMessage {
    UserActivityByTypeQuery(UserActivityByTypeQuery),
}

#[async_trait]
impl MessageResponder for UserQueriesMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            UserQueriesMessage::ActiveAuthorsQuery(message) => message.handle(connection).await,
            UserQueriesMessage::ActiveReviewersQuery(message) => message.handle(connection).await,
        }
    }
}

#[async_trait]
impl MessageResponder for UserMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            UserMessage::UserActivityByTypeQuery(message) => message.handle(connection).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAuthorsQuery {}

#[async_trait]
impl MessageResponder for ActiveAuthorsQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let active_authors = match connection {
            Connection::Pool(pool) => User::fetch_active_authors(pool).await,
            Connection::Transaction(transaction) => {
                User::fetch_active_authors_via_transaction(transaction).await
            }
        };
        match active_authors {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/user/active-authors: {:?}", e);
                match e {
                    UserError::DatabaseError { .. } => HttpResponse::InternalServerError().finish(),
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveReviewersQuery {}

#[async_trait]
impl MessageResponder for ActiveReviewersQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let active_reviewers = match connection {
            Connection::Pool(pool) => User::fetch_active_reviewers(pool).await,
            Connection::Transaction(transaction) => {
                User::fetch_active_reviewers_via_transaction(transaction).await
            }
        };
        match active_reviewers {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/user/active-reviewers: {:?}", e);
                match e {
                    UserError::DatabaseError { .. } => HttpResponse::InternalServerError().finish(),
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserActivityByTypeQuery {
    user_id: i32,
}

#[async_trait]
impl MessageResponder for UserActivityByTypeQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let activity = match connection {
            Connection::Pool(pool) => User::fetch_activity_by_type(self.user_id, pool).await,
            Connection::Transaction(transaction) => {
                User::fetch_activity_by_type(self.user_id, transaction).await
            }
        };
        match activity {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(data),
            Err(e) => {
                println!("/user/activity-by-type: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
