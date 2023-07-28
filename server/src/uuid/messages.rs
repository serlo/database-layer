use crate::operation::{self, Operation};
use actix_web::HttpResponse;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::model::{Uuid, UuidFetcher};
use crate::message::MessageResponder;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum UuidMessage {
    UuidQuery(uuid_query::Payload),
    UuidSetStateMutation(uuid_set_state_mutation::Payload),
}

#[async_trait]
impl MessageResponder for UuidMessage {
    #[allow(clippy::async_yields_async)]
    async fn handle<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
        &self,
        acquire_from: A,
    ) -> HttpResponse {
        match self {
            UuidMessage::UuidQuery(message) => message.handle(acquire_from).await,
            UuidMessage::UuidSetStateMutation(message) => message.handle(acquire_from).await,
        }
    }
}

pub mod uuid_query {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub id: i32,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Uuid;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Ok(Uuid::fetch(self.id, acquire_from).await?)
        }
    }
}

pub mod uuid_set_state_mutation {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Payload {
        pub ids: Vec<i32>,
        pub user_id: i32,
        pub trashed: bool,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Output {
        success: bool,
    }

    #[async_trait]
    impl Operation for Payload {
        type Output = Output;

        async fn execute<'e, A: sqlx::Acquire<'e, Database = sqlx::MySql> + std::marker::Send>(
            &self,
            acquire_from: A,
        ) -> operation::Result<Self::Output> {
            Uuid::set_uuid_state(self, acquire_from).await?;
            Ok(Output { success: true })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::create_database_pool;
    use crate::uuid::model::{Uuid, UuidError, UuidFetcher};

    #[actix_rt::test]
    async fn solution_is_null_for_math_puzzle() {
        let pool = create_database_pool().await.unwrap();
        let mut transaction = pool.begin().await.unwrap();

        // Add solution with id 100000 and math-puzzle with id 100001
        sqlx::query(
            r#"
                INSERT INTO uuid (id, trashed, discriminator)
                VALUES (100000, 0, 'entity'),
                       (100001, 0, 'entity')
            "#,
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
        sqlx::query(
            r#"
                INSERT INTO entity (id, type_id, instance_id, license_id, date, current_revision_id)
                VALUES (100000, 2, 1, 1, CURDATE(), 1),
                       (100001, 39, 1, 1, CURDATE(), 2)
            "#,
        )
        .execute(&mut *transaction)
        .await
        .unwrap();

        // Link solution and math-puzzle
        sqlx::query(
            r#"
                INSERT INTO entity_link (parent_id, child_id, type_id)
                VALUES (100001, 100000, 9)
            "#,
        )
        .execute(&mut *transaction)
        .await
        .unwrap();

        let result = Uuid::fetch(100000, &mut *transaction).await;

        assert!(result.is_err());
        match result.as_ref().err().unwrap() {
            UuidError::EntityMissingRequiredParent => (),
            _ => panic!("Error does not fulfill assertions {:?}", result.err()),
        }
    }
}
