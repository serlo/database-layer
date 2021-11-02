#[cfg(test)]
mod tests {
    use test_utils::*;

    #[actix_rt::test]
    async fn start_thread_rejects_when_content_is_empty() {
        let response = Message::new(
            "ThreadCreateThreadMutation",
            json!({
                "title": "title",
                "content": "",
                "objectId": 1565,
                "userId": 1,
                "subscribe": true,
                "sendEmail": false,
            }),
        )
        .execute()
        .await;

        assert_bad_request(response, "Cannot create thread: content is empty").await;
    }

    #[actix_rt::test]
    async fn create_comment_rejects_when_content_is_empty() {
        let response = Message::new(
            "ThreadCreateCommentMutation",
            json!({
                "threadId": 17774,
                "userId": 1,
                "content": "",
                "subscribe": true,
                "sendEmail": false,
            }),
        )
        .execute()
        .await;

        assert_bad_request(response, "Cannot create comment: content is empty").await;
    }
}
