use async_trait::async_trait;

use serde::Serialize;

use super::{ConcreteUuid, Uuid, UuidError, UuidFetcher};

use crate::datetime::DateTime;
use crate::format_alias;

use std::str::FromStr;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub author_id: i32,
    pub title: Option<String>,
    pub date: DateTime,
    pub archived: bool,
    pub content: String,
    pub parent_id: i32,
    pub children_ids: Vec<i32>,
    pub status: CommentStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommentStatus {
    NoStatus,
    Open,
    Done,
}

impl FromStr for CommentStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "no_status" => Ok(CommentStatus::NoStatus),
            "open" => Ok(CommentStatus::Open),
            "done" => Ok(CommentStatus::Done),
            _ => Err(()),
        }
    }
}

macro_rules! fetch_one_comment {
    ($id: expr, $executor: expr) => {
       sqlx::query!(
            r#"
                SELECT u.trashed, c.author_id, c.title, c.date, c.archived, c.content, c.parent_id,
                        c.uuid_id, p.title as parent_title, comment_status.name as status
                    FROM comment c
                    LEFT JOIN comment p ON p.id = c.parent_id
                    LEFT JOIN comment_status ON comment_status.id = c.comment_status_id
                    JOIN uuid u ON u.id = c.id
                    WHERE c.id = ?
            "#,
            $id
        )
        .fetch_one($executor)
    };
}

macro_rules! fetch_all_children {
    ($id: expr, $executor: expr) => {
        sqlx::query!(
            r#"
                SELECT id
                    FROM comment
                    WHERE parent_id = ?
            "#,
            $id
        )
        .fetch_all($executor)
    };
}

macro_rules! to_comment {
    ($id: expr, $comment: expr, $children: expr, $context: expr) => {{
        let comment = $comment.map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let children = $children?;
        let context = $context?;

        Ok(Uuid {
            id: $id,
            trashed: comment.trashed != 0,
            alias: format_alias(
                context.as_deref(),
                $id,
                Some(
                    comment
                        .title
                        .as_ref()
                        .or_else(|| comment.parent_title.as_ref())
                        .unwrap_or(&format!("{}", $id))
                        .as_str(),
                ),
            ),
            concrete_uuid: ConcreteUuid::Comment(Comment {
                __typename: "Comment".to_string(),
                author_id: comment.author_id as i32,
                title: comment.title,
                status: comment
                    .status
                    .and_then(|status| status.parse().ok())
                    .unwrap_or(CommentStatus::NoStatus),
                date: comment.date.into(),
                archived: comment.archived != 0,
                content: comment.content.unwrap_or_else(|| "".to_string()),
                parent_id: comment.parent_id.or(comment.uuid_id).unwrap() as i32,
                children_ids: children.iter().map(|child| child.id as i32).collect(),
            }),
        })
    }};
}

#[async_trait]
impl UuidFetcher for Comment {
    async fn fetch<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql> + std::marker::Send>(
        id: i32,
        acquire_from: A,
    ) -> Result<Uuid, UuidError> {
        let mut transaction = acquire_from.begin().await?;
        let comment = fetch_one_comment!(id, &mut *transaction).await;
        let children = fetch_all_children!(id, &mut *transaction).await;
        let context = Comment::fetch_context(id, &mut *transaction).await;

        transaction.commit().await?;

        to_comment!(id, comment, children, context)
    }
}

impl Comment {
    pub async fn fetch_context<'a, A: sqlx::Acquire<'a, Database = sqlx::MySql>>(
        id: i32,
        acquire_from: A,
    ) -> Result<Option<String>, UuidError> {
        let mut transaction = acquire_from.begin().await?;
        let object = sqlx::query!(
            r#"
                SELECT uuid_id as id
                    FROM (
                        SELECT id, uuid_id FROM comment c
                        UNION ALL
                        SELECT c.id, p.uuid_id FROM comment p LEFT JOIN comment c ON c.parent_id = p.id
                    ) t
                    WHERE id = ? AND uuid_id IS NOT NULL
            "#,
            id
        )
        .fetch_one(&mut *transaction).await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => UuidError::NotFound,
            error => error.into(),
        })?;
        let context = Uuid::fetch_context(object.id.unwrap() as i32, &mut *transaction).await?;
        transaction.commit().await?;
        Ok(context)
    }
}
