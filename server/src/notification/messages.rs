use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{
    Notifications, NotificationsError, SetNotificationStateError, SetNotificationStatePayload,
};
use crate::database::Connection;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum NotificationMessage {
    NotificationsQuery(notifications_query::Payload),
    NotificationSetStateMutation(NotificationSetStateMutation),
}

#[async_trait]
impl MessageResponder for NotificationMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle(&self, connection: Connection<'_, '_>) -> HttpResponse {
        match self {
            NotificationMessage::NotificationsQuery(payload) => {
                payload.handle("NotificationsQuery", connection).await
            }
            NotificationMessage::NotificationSetStateMutation(message) => {
                message.handle(connection).await
            }
        }
    }
}

pub mod notifications_query {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Notifications;

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(match connection {
                Connection::Pool(pool) => Notifications::fetch(self.user_id, pool).await?,
                Connection::Transaction(transaction) => {
                    Notifications::fetch_via_transaction(self.user_id, transaction).await?
                }
            })
        }
    }

    impl From<NotificationsError> for operation::Error {
        fn from(e: NotificationsError) -> Self {
            match e {
                NotificationsError::DatabaseError { .. } => {
                    operation::Error::InternalServerError { error: Box::new(e) }
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
