pub use model::*;
pub use routes::init;

mod model;
mod routes;

#[cfg(test)]
pub(crate) mod test_helpers {
    use chrono::Duration;

    use super::Event;
    use crate::database::Executor;
    use crate::datetime::DateTime;

    pub(crate) async fn fetch_age_of_newest_event<'a, E>(
        object_id: i32,
        executor: E,
    ) -> Result<Duration, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await.unwrap();

        let result = sqlx::query!(
            r#"SELECT id FROM event_log WHERE uuid_id = ? ORDER BY date DESC"#,
            object_id
        )
        .fetch_one(&mut transaction)
        .await;

        match result {
            Ok(event) => {
                let event = Event::fetch_via_transaction(event.id as i32, &mut transaction)
                    .await
                    .unwrap();
                Ok(DateTime::now().signed_duration_since(event.abstract_event.date))
            }
            Err(sqlx::Error::RowNotFound) => Ok(Duration::max_value()),
            Err(inner) => Err(inner),
        }
    }
}
