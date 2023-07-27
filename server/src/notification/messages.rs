use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::Notifications;
use crate::message::MessageResponder;
use crate::operation::{self, Operation, SuccessOutput};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum NotificationMessage {
    NotificationsQuery(notifications_query::Payload),
    NotificationSetStateMutation(set_state_mutation::Payload),
}

#[async_trait]
impl MessageResponder for NotificationMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        self.handle(acquire_from).await
    }
}

pub mod notifications_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Notifications;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Notifications::fetch(self.user_id, acquire_from).await?)
        }
    }
}

pub mod set_state_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub ids: Vec<i32>,
        pub user_id: i32,
        pub unread: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = SuccessOutput;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Notifications::set_notification_state(self, acquire_from).await?;
            Ok(SuccessOutput { success: true })
        }
    }
}
