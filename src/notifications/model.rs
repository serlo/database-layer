use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifications {
    pub user_id: i32,
    pub notifications: Vec<Notification>,
}

#[derive(Error, Debug)]
pub enum NotificationsError {
    #[error("Navigation cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub unread: bool,
    pub event_id: i32,
}

impl Notifications {
    pub async fn fetch(
        user_id: i32,
        pool: &MySqlPool,
    ) -> Result<Notifications, NotificationsError> {
        let notifications_fut = sqlx::query!(
            r#"
                SELECT n.id, n.seen, e.event_log_id
                    FROM notification n
                    JOIN notification_event e ON n.id = e.notification_id
                    WHERE n.user_id = ?
                    ORDER BY n.date DESC, n.id DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|inner| NotificationsError::DatabaseError { inner })?;

        let notifications: Vec<Notification> = notifications_fut
            .iter()
            .map(|child| Notification {
                id: child.id as i32,
                unread: child.seen == 0,
                event_id: child.event_log_id as i32,
            })
            .collect();

        Ok(Notifications {
            user_id,
            notifications,
        })
    }
}
