use actix_web::body::to_bytes;
use actix_web::HttpResponse;
use rand::{distributions::Alphanumeric, Rng};
use serde_json::{from_slice, from_value, json, Value};
use server::create_database_pool;
use server::database::Connection;
use server::message::{Message, MessageResponder};

pub async fn begin_transaction<'a>() -> sqlx::Transaction<'a, sqlx::MySql> {
    create_database_pool().await.unwrap().begin().await.unwrap()
}

pub async fn handle_message<'a>(
    transaction: &mut sqlx::Transaction<'a, sqlx::MySql>,
    message_type: &str,
    payload: Value,
) -> HttpResponse {
    let message = json!({ "type": message_type, "payload": payload });
    let message = from_value::<Message>(message).unwrap();
    message.handle(Connection::Transaction(transaction)).await
}

pub async fn assert_ok_response(response: HttpResponse, expected_result: Value) {
    assert!(response.status().is_success());

    let body = to_bytes(response.into_body()).await.unwrap();
    let result: Value = from_slice(&body).unwrap();
    assert_eq!(result, expected_result);
}

pub async fn create_new_test_user<'a, E>(executor: E) -> Result<i32, sqlx::Error>
where
    E: sqlx::Acquire<'a, Database = sqlx::MySql>,
{
    let mut transaction = executor.begin().await?;

    sqlx::query!(
        r#"
                INSERT INTO uuid (trashed, discriminator) VALUES (0, "user")
            "#
    )
    .execute(&mut transaction)
    .await?;

    let new_user_id = sqlx::query!("SELECT LAST_INSERT_ID() as id FROM uuid")
        .fetch_one(&mut transaction)
        .await?
        .id as i32;

    sqlx::query!(
        r#"
                INSERT INTO user (id, username, email, password, token)
                VALUES (?, ?, ?, ?, ?)
            "#,
        new_user_id,
        random_string(10),
        random_string(10),
        "",
        random_string(10)
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(new_user_id)
}

pub async fn set_description<'a, E>(
    user_id: i32,
    description: &str,
    executor: E,
) -> Result<(), sqlx::Error>
where
    E: sqlx::mysql::MySqlExecutor<'a>,
{
    sqlx::query!(
        "update user set description = ? where id = ?",
        description,
        user_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn set_entity_revision_field<'a>(
    revision_id: i32,
    field: &str,
    value: &str,
    executor: impl sqlx::Acquire<'a, Database = sqlx::MySql>,
) -> Result<(), sqlx::Error> {
    let mut transaction = executor.begin().await?;

    if sqlx::query!(
        "update entity_revision_field set value = ? where id = ? and field = ?",
        value,
        revision_id,
        value
    )
    .execute(&mut transaction)
    .await?
    .rows_affected()
        == 0
    {
        sqlx::query!(
            "insert into entity_revision_field (entity_revision_id, field, value) values (?, ?, ?)",
            revision_id,
            field,
            value
        )
        .execute(&mut transaction)
        .await?;
    };
    transaction.commit().await?;
    Ok(())
}

fn random_string(nr: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(nr)
        .map(char::from)
        .collect()
}
