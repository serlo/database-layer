/// This trait should be used for functions that can accept either `&sqlx::MySqlPool` or `&mut Transaction<MySql>`.
///
/// # Defining a function accepting `Executor`
///
/// If you have on query, you can use `executor` directly:
///
/// ```rust
/// use server::database::Executor;
///
/// async fn fetch_via_transaction<'a, E>(executor: E) -> Result<(), sqlx::Error>
/// where
///     E: Executor<'a>,
/// {
///     let events = sqlx::query!(r#"SELECT id FROM event_log"#).fetch_all(executor).await?;
///     Ok(())
/// }
/// ```
///
/// If you have more than one query, you'll need to start a new transaction:
///
/// ```rust
/// use server::database::Executor;
///
/// async fn fetch_via_transaction<'a, E>(executor: E) -> Result<(), sqlx::Error>
/// where
///     E: Executor<'a>,
/// {
///     let mut transaction = executor.begin().await?;
///     let events = sqlx::query!(r#"SELECT id FROM event_log"#).fetch_all(&mut *transaction).await?;
///     let users = sqlx::query!(r#"SELECT id FROM user"#).fetch_all(&mut *transaction).await?;
///     transaction.commit().await?;
///     Ok(())
/// }
/// ```
///
/// Note: you can't parallelize multiple queries using `try_join` in this case because Rust
/// will consider `executor` as moved. For this reason, we write our models using `MySqlPool`
/// by default instead and only provide an unoptimized `Executor`-variant (suffixed by `_via_transaction`)
/// if we need it in a mutation or in tests.
pub trait Executor<'a>:
    sqlx::Executor<'a, Database = sqlx::MySql> + sqlx::Acquire<'a, Database = sqlx::MySql>
{
}

impl<'a, E> Executor<'a> for E where
    E: sqlx::Executor<'a, Database = sqlx::MySql> + sqlx::Acquire<'a, Database = sqlx::MySql>
{
}
