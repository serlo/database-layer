use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{
    Notifications, NotificationsError, SetNotificationStateError, SetNotificationStatePayload,
};
use crate::database::Connection;
use crate::message::MessageResponder;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum NotificationMessage {
    NotificationsQuery(NotificationsQuery),
    NotificationSetStateMutation(NotificationSetStateMutation),
}

#[async_trait]
impl MessageResponder for NotificationMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            NotificationMessage::NotificationsQuery(message) => message.handle(connection).await,
            NotificationMessage::NotificationSetStateMutation(message) => {
                message.handle(connection).await
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsQuery {
    pub user_id: i32,
}

#[async_trait]
impl MessageResponder for NotificationsQuery {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let notifications = match connection {
            Connection::Pool(pool) => Notifications::fetch(self.user_id, pool).await,
            Connection::Transaction(transaction) => {
                Notifications::fetch_via_transaction(self.user_id, transaction).await
            }
        };
        match notifications {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/notifications/{}: {:?}", self.user_id, e);
                match e {
                    NotificationsError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSetStateMutation {
    pub ids: Vec<i32>,
    pub user_id: i32,
    pub unread: bool,
}

#[async_trait]
impl MessageResponder for NotificationSetStateMutation {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        let payload = SetNotificationStatePayload {
            ids: self.ids.clone(),
            user_id: self.user_id,
            unread: self.unread,
        };
        let response = match connection {
            Connection::Pool(pool) => Notifications::set_notification_state(payload, pool).await,
            Connection::Transaction(transaction) => {
                Notifications::set_notification_state(payload, transaction).await
            }
        };
        match response {
            Ok(data) => HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(&data),
            Err(e) => {
                println!("/set-notification-state: {:?}", e);
                match e {
                    SetNotificationStateError::DatabaseError { .. } => {
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    }
}
