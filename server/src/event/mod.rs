pub use messages::EventMessage;
pub use model::*;

mod messages;
mod model;

#[cfg(test)]
pub(crate) mod test_helpers {
    use chrono::Duration;

    use super::Event;
    use crate::datetime::DateTime;

    pub(crate) async fn fetch_age_of_newest_event<
        'a,
        A: sqlx::Acquire<'a, Database = sqlx::MySql>,
    >(
        object_id: i32,
        acquire_from: A,
    ) -> Result<Duration, sqlx::Error> {
        let mut transaction = acquire_from.begin().await.unwrap();

        let result = sqlx::query!(
            r#"SELECT id FROM event_log WHERE uuid_id = ? ORDER BY date DESC"#,
            object_id
        )
        .fetch_one(&mut *transaction)
        .await;

        match result {
            Ok(event) => {
                let event = Event::fetch_via_transaction(event.id as i32, &mut *transaction)
                    .await
                    .unwrap();
                Ok(DateTime::now().signed_duration_since(event.abstract_event.date))
            }
            Err(sqlx::Error::RowNotFound) => Ok(Duration::max_value()),
            Err(inner) => Err(inner),
        }
    }
}
