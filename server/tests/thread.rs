#[cfg(test)]
mod start_thread_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn fails_when_content_is_empty() {
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

        assert_bad_request(response, "content is empty").await;
    }
}

#[cfg(test)]
mod create_comment_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn fails_when_content_is_empty() {
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

        assert_bad_request(response, "content is empty").await;
    }
}
