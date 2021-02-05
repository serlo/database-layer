use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::model::{User, UserError};
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UserMessage {
    ActiveAuthorsQuery(ActiveAuthorsQuery),
    ActiveReviewersQuery(ActiveReviewersQuery),
}

#[async_trait]
impl MessageResponder for UserMessage {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match self {
            UserMessage::ActiveAuthorsQuery(message) => message.handle(pool).await,
            UserMessage::ActiveReviewersQuery(message) => message.handle(pool).await,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAuthorsQuery {}

#[async_trait]
impl MessageResponder for ActiveAuthorsQuery {
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match User::fetch_active_authors(pool).await {
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
    async fn handle(&self, pool: &MySqlPool) -> HttpResponse {
        match User::fetch_active_reviewers(pool).await {
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
