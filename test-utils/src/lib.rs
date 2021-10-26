use rand::{distributions::Alphanumeric, Rng};

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
        "test_user",
        random_string(10)
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(new_user_id)
}

pub async fn delete_all_test_user<'a, E>(executor: E) -> Result<(), sqlx::Error>
where
    E: sqlx::mysql::MySqlExecutor<'a>,
{
    sqlx::query!(r#"delete from user where user.password = "test_user""#)
        .execute(executor)
        .await?;
    Ok(())
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
