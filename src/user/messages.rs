use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{User, UserError};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum UserMessage {
    ActiveAuthorsQuery(ActiveAuthorsQuery),
    ActiveReviewersQuery(ActiveReviewersQuery),
}

#[async_trait]
impl MessageResponder for UserMessage {
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            UserMessage::ActiveAuthorsQuery(message) => message.handle(connection).await,
            UserMessage::ActiveReviewersQuery(message) => message.handle(connection).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAuthorsQuery {}

#[async_trait]
impl MessageResponder for ActiveAuthorsQuery {
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
