use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::User;
use crate::message::MessageResponder;
use crate::operation::{self, Operation};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UserMessage {
    ActiveAuthorsQuery(Option<serde_json::Value>),
    ActiveReviewersQuery(Option<serde_json::Value>),
    // TODO: Delete when not needed any more in api.serlo.org
    // See https://github.com/serlo/api.serlo.org/issues/459
    ActivityByTypeQuery(user_activity_by_type_query::Payload),
    UserActivityByTypeQuery(user_activity_by_type_query::Payload),
    UserAddRoleMutation(user_add_role_mutation::Payload),
    UserCreateMutation(user_create_mutation::Payload),
    UserDeleteBotsMutation(user_delete_bots_mutation::Payload),
    UserDeleteRegularUsersMutation(user_delete_regular_users_mutation::Payload),
    UserPotentialSpamUsersQuery(potential_spam_users_query::Payload),
    UserRemoveRoleMutation(user_remove_role_mutation::Payload),
    UsersByRoleQuery(users_by_role_query::Payload),
    UserSetDescriptionMutation(user_set_description_mutation::Payload),
    UserSetEmailMutation(user_set_email_mutation::Payload),
}

#[async_trait]
impl MessageResponder for UserMessage {
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            UserMessage::ActiveAuthorsQuery(_) => {
                active_authors_query::Payload {}.handle(acquire_from).await
            }
            UserMessage::ActiveReviewersQuery(_) => {
                active_reviewers_query::Payload {}
                    .handle(acquire_from)
                    .await
            }
            UserMessage::ActivityByTypeQuery(payload) => payload.handle(acquire_from).await,
            UserMessage::UserActivityByTypeQuery(payload) => payload.handle(acquire_from).await,
            UserMessage::UserAddRoleMutation(payload) => payload.handle(acquire_from).await,
            UserMessage::UserCreateMutation(payload) => payload.handle(acquire_from).await,
            UserMessage::UserDeleteBotsMutation(payload) => payload.handle(acquire_from).await,
            UserMessage::UserDeleteRegularUsersMutation(payload) => {
                payload.handle(acquire_from).await
            }
            UserMessage::UserPotentialSpamUsersQuery(payload) => payload.handle(acquire_from).await,
            UserMessage::UserRemoveRoleMutation(payload) => payload.handle(acquire_from).await,
            UserMessage::UsersByRoleQuery(payload) => payload.handle(acquire_from).await,
            UserMessage::UserSetDescriptionMutation(payload) => payload.handle(acquire_from).await,
            UserMessage::UserSetEmailMutation(payload) => payload.handle(acquire_from).await,
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(User::fetch_active_authors(acquire_from).await?)
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(User::fetch_active_reviewers(acquire_from).await?)
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

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(User::fetch_activity_by_type(self.user_id, acquire_from).await?)
        }
    }
}

pub mod user_add_role_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub username: String,
        pub role_name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            User::add_role(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}

pub mod user_create_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub username: String,
        pub email: String,
        pub password: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
        pub user_id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let user_id: i32 = User::create(self, acquire_from).await?;
            Ok(Output {
                success: true,
                user_id,
            })
        }
    }
}

pub mod user_delete_bots_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub bot_ids: Vec<i32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
        pub email_hashes: Vec<String>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let email_hashes = User::delete_bot(self, acquire_from).await?;
            Ok(Output {
                success: true,
                email_hashes,
            })
        }
    }
}

pub mod user_delete_regular_users_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            User::delete_regular_user(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}

pub mod potential_spam_users_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub first: i32,
        pub after: Option<i32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub user_ids: Vec<i32>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            if self.first > 10_000 {
                return Err(operation::Error::BadRequest {
                    reason: "parameter `first` is too high".to_string(),
                });
            };
            Ok(Output {
                user_ids: User::potential_spam_users(self, acquire_from).await?,
            })
        }
    }
}

pub mod user_remove_role_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub username: String,
        pub role_name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            User::remove_role(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}

pub mod users_by_role_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub role_name: String,
        pub first: i32,
        pub after: Option<i32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub users_by_role: Vec<i32>,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Output {
                users_by_role: User::users_by_role(self, acquire_from).await?,
            })
        }
    }
}

pub mod user_set_description_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub description: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            User::set_description(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}

pub mod user_set_email_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub user_id: i32,
        pub email: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        pub success: bool,
        pub username: String,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            let username = User::set_email(self, acquire_from).await?;
            Ok(Output {
                success: true,
                username,
            })
        }
    }
}
