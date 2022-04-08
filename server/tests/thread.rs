#[cfg(test)]
mod all_threads_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_list_of_thread_ids() {
        let response = Message::new("AllThreadsQuery", json!({ "first": 5 }))
            .execute()
            .await;

        assert_ok(
            response,
            json!({ "firstCommentIds": [35435, 35361, 35183, 35163, 35090] }),
        )
        .await;
    }

    #[actix_rt::test]
    async fn with_parameter_after() {
        let response = Message::new("AllThreadsQuery", json!({ "first": 3, "after": 35361 }))
            .execute()
            .await;

        assert_ok(
            response,
            json!({ "firstCommentIds": [35183, 35163, 35090] }),
        )
        .await;
    }
}

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
