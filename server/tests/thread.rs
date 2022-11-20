mod all_threads_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_list_of_thread_ids() {
        Message::new("AllThreadsQuery", json!({ "first": 5 }))
            .execute()
            .await
            .should_be_ok_with_body(
                json!({ "firstCommentIds": [34546, 35163, 35435, 35361, 34119] }),
            );
    }

    #[actix_rt::test]
    async fn orders_threads_by_last_commented() {
        let mut transaction = begin_transaction().await;

        Message::new(
            "ThreadCreateCommentMutation",
            json!({
                "threadId": 34119,
                "userId": 1,
                "content": "last comment",
                "subscribe": true,
                "sendEmail": false,
            }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();

        Message::new("AllThreadsQuery", json!({ "first": 5 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with_body(
                json!({ "firstCommentIds": [34119, 34546, 35163, 35435, 35361] }),
            );
    }

    #[actix_rt::test]
    async fn with_parameter_after_maintaining_pagination_order() {
        Message::new(
            "AllThreadsQuery",
            json!({ "first": 10, "after": "2015-02-26T12:48:59+01:00" }),
        )
        .execute()
        .await
        .should_be_ok_with_body(json!({ "firstCommentIds": [35163, 35435, 35361, 34119, 35090, 35085, 26976, 35083, 35082, 30251] }));

        Message::new(
            "AllThreadsQuery",
            json!({ "first": 10, "after": "2015-02-19T16:47:16+01:00" }),
        )
        .execute()
        .await
        .should_be_ok_with_body(json!({ "firstCommentIds": [35085, 26976, 35083, 35082, 30251, 35073, 34618, 34793, 34539, 34095] }));

        Message::new(
            "AllThreadsQuery",
            json!({ "first": 5, "after": "2015-02-19T15:52:05+01:00" }),
        )
        .execute()
        .await
        .should_be_ok_with_body(json!({ "firstCommentIds": [35073, 34618, 34793, 34539, 34095] }));
    }

    #[actix_rt::test]
    async fn with_parameter_instance() {
        Message::new("AllThreadsQuery", json!({ "first": 3, "instance": "en" }))
            .execute()
            .await
            .should_be_ok_with_body(json!({ "firstCommentIds": [] }));
    }

    #[actix_rt::test]
    async fn does_not_return_threads_on_user_page() {
        Message::new(
            "AllThreadsQuery",
            json!({ "first": 1, "after": "2014-08-05T16:47:21+01:00" }),
        )
        .execute()
        .await
        .should_be_ok_with(|body| assert_ne!(body["firstCommentIds"][0], 27053));
    }
}

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
