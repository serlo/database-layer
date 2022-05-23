#[cfg(test)]
mod all_threads_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_list_of_thread_ids() {
        Message::new("AllThreadsQuery", json!({ "first": 5 }))
            .execute()
            .await
            .should_be_ok_with_body(
                json!({ "firstCommentIds": [35435, 35361, 35183, 35163, 35090] }),
            );
    }

    #[actix_rt::test]
    async fn with_parameter_after() {
        Message::new("AllThreadsQuery", json!({ "first": 3, "after": 35361 }))
            .execute()
            .await
            .should_be_ok_with_body(json!({ "firstCommentIds": [35183, 35163, 35090] }));
    }
}

#[cfg(test)]
mod start_thread_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn fails_when_content_is_empty() {
        Message::new(
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
        .await
        .should_be_bad_request();
    }
}

#[cfg(test)]
mod create_comment_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn fails_when_content_is_empty() {
        Message::new(
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
        .await
        .should_be_bad_request();
    }
}
