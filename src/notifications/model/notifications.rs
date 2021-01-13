use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifications {
    pub user_id: i32,
    pub notifications: Vec<Notification>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub unread: bool,
    pub event_id: i32,
}

impl Notifications {
    pub async fn get_notifications_for_user(
        user_id: i32,
        pool: &MySqlPool,
    ) -> Result<Notifications> {
        //todo: put in helper (is_valid_user or smth)
        sqlx::query!(
            r#"SELECT id FROM uuid WHERE discriminator = "user" AND id = ?"#,
            user_id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                anyhow::Error::new(NotificationsError::NotFound { user_id })
            }
            e => anyhow::Error::new(e),
        })?;

        let notifications_fut = sqlx::query!(
            "
            SELECT notification.id, notification.seen, notification_event.event_log_id FROM notification
            INNER JOIN notification_event ON notification.id = notification_event.notification_id
            WHERE notification.user_id = ?
            ORDER BY notification.date DESC, notification.id DESC
            ",
            user_id
        )
        .fetch_all(pool)
        .await?;

        let notifications: Vec<Notification> = notifications_fut
            .iter()
            .map(|child| Notification {
                id: child.id as i32,
                unread: child.seen == 0,
                event_id: child.event_log_id as i32,
            })
            .collect();

        Ok(Notifications {
            user_id: user_id,
            notifications: notifications,
        })
    }
}

#[derive(Error, Debug)]
pub enum NotificationsError {
    #[error("Given id {user_id:?} is not a valid user id.")]
    NotFound { user_id: i32 },
}
