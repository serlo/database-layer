use anyhow::Result;
use futures::try_join;
use serde::Serialize;
use sqlx::MySqlPool;

use super::Uuid;
use crate::{format_alias, format_datetime};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub trashed: bool,
    pub alias: String,
    pub author_id: i32,
    pub title: Option<String>,
    pub date: String,
    pub archived: bool,
    pub content: String,
    pub parent_id: i32,
    pub children_ids: Vec<i32>,
}

impl Comment {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Comment> {
        let comment_fut = sqlx::query!(
            r#"
                SELECT u.trashed, c.author_id, c.title, c.date, c.archived, c.content, c.parent_id, c.uuid_id, p.title as parent_title
                    FROM comment c
                    LEFT JOIN comment p ON p.id = c.parent_id
                    JOIN uuid u ON u.id = c.id
                    WHERE c.id = ?
            "#,
            id
        )
        .fetch_one(pool);
        let children_fut = sqlx::query!(
            r#"
                SELECT id
                    FROM comment
                    WHERE parent_id = ?
            "#,
            id
        )
        .fetch_all(pool);
        let (comment, children) = try_join!(comment_fut, children_fut)?;

        Ok(Comment {
            __typename: String::from("Comment"),
            id,
            trashed: comment.trashed != 0,
            alias: format_alias(
                Self::find_context_by_id(id, pool).await?.as_deref(),
                id,
                Some(
                    comment
                        .title
                        .as_ref()
                        .or_else(|| comment.parent_title.as_ref())
                        .unwrap_or(&format!("{}", id))
                        .as_str(),
                ),
            ),
            author_id: comment.author_id as i32,
            title: comment.title,
            date: format_datetime(&comment.date),
            archived: comment.archived != 0,
            content: comment.content.unwrap_or_else(|| String::from("")),
            parent_id: comment.parent_id.or(comment.uuid_id).unwrap() as i32,
            children_ids: children.iter().map(|child| child.id as i32).collect(),
        })
    }

    pub async fn find_context_by_id(
        id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<String>, sqlx::Error> {
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
        ).fetch_one(pool).await?;
        Uuid::find_context_by_id(object.id.unwrap() as i32, pool).await
    }
}
