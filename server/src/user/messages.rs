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
            UserMessage::UserAddRoleMutation(payload) => {
                payload.handle("UserAddRoleMutation", connection).await
            }
            UserMessage::UserCreateMutation(payload) => {
                payload.handle("UserCreateMutation", connection).await
            }
            UserMessage::UserDeleteBotsMutation(payload) => {
                payload.handle("UserDeleteBotsMutation", connection).await
            }
            UserMessage::UserDeleteRegularUsersMutation(payload) => {
                payload
                    .handle("UserDeleteRegularUsersMutation", connection)
                    .await
            }
            UserMessage::UserPotentialSpamUsersQuery(payload) => {
                payload
                    .handle("UserPotentialSpamUsersQuery", connection)
                    .await
            }
            UserMessage::UserRemoveRoleMutation(payload) => {
                payload.handle("UserRemoveRoleMutation", connection).await
            }
            UserMessage::UsersByRoleQuery(payload) => {
                payload.handle("UsersByRoleQuery", connection).await
            }
            UserMessage::UserSetDescriptionMutation(payload) => {
                payload
                    .handle("UserSetDescriptionMutation", connection)
                    .await
            }
            UserMessage::UserSetEmailMutation(payload) => {
                payload.handle("UserSetEmailMutation", connection).await
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

pub mod user_add_role {
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => User::add_role(self, pool).await?,
                Connection::Transaction(transaction) => User::add_role(self, transaction).await?,
            };
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let user_id: i32 = match connection {
                Connection::Pool(pool) => User::create(self, pool).await?,
                Connection::Transaction(transaction) => User::create(self, transaction).await?,
            };
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let email_hashes = match connection {
                Connection::Pool(pool) => User::delete_bot(self, pool).await?,
                Connection::Transaction(transaction) => User::delete_bot(self, transaction).await?,
            };
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => User::delete_regular_user(self, pool).await?,
                Connection::Transaction(transaction) => {
                    User::delete_regular_user(self, transaction).await?
                }
            };

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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            if self.first > 10_000 {
                return Err(operation::Error::BadRequest {
                    reason: "parameter `first` is too high".to_string(),
                });
            };
            Ok(Output {
                user_ids: match connection {
                    Connection::Pool(pool) => User::potential_spam_users(self, pool).await?,
                    Connection::Transaction(transaction) => {
                        User::potential_spam_users(self, transaction).await?
                    }
                },
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => User::remove_role(self, pool).await?,
                Connection::Transaction(transaction) => {
                    User::remove_role(self, transaction).await?
                }
            };
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            Ok(Output {
                users_by_role: match connection {
                    Connection::Pool(pool) => User::users_by_role(self, pool).await?,
                    Connection::Transaction(transaction) => {
                        User::users_by_role(self, transaction).await?
                    }
                },
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            match connection {
                Connection::Pool(pool) => User::set_description(self, pool).await?,
                Connection::Transaction(transaction) => {
                    User::set_description(self, transaction).await?
                }
            };
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

        async fn execute(&self, connection: Connection<'_, '_>) -> operation::Result<Self::Output> {
            let username = match connection {
                Connection::Pool(pool) => User::set_email(self, pool).await?,
                Connection::Transaction(transaction) => User::set_email(self, transaction).await?,
            };
            Ok(Output {
                success: true,
                username,
            })
        }
    }
}
