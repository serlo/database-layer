mod all_threads_query {
    use std::{thread, time::Duration};
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

        // it was sometimes not persisting fast enough the mutation above
        thread::sleep(Duration::from_millis(1000));

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

mod thread_mutations {
    use actix_web::http::StatusCode;
    use rstest::*;
    use test_utils::*;

    #[rstest]
    // positive cases:
    // start thread under user
    #[case(StatusCode::OK, 1, 1, "Title", "This is content.", true, false)]
    // start thread under article
    #[case(StatusCode::OK, 1565, 1, "Title", "This is content.", true, false)]
    // negative cases:
    // valid payload except content is empty
    #[case(StatusCode::BAD_REQUEST, 17774, 1, "Title", "", true, false)]
    // valid payload except UUID does not exist
    #[case(
        StatusCode::BAD_REQUEST,
        999999,
        1,
        "Title",
        "This is content.",
        true,
        false
    )]
    #[actix_rt::test]
    async fn create_thread(
        #[case] expected_response: StatusCode,
        #[case] object_id: i32,
        #[case] user_id: i32,
        #[case] title: &str,
        #[case] content: &str,
        #[case] subscribe: bool,
        #[case] send_email: bool,
    ) {
        let mut transaction = begin_transaction().await;

        let result = Message::new(
            "ThreadCreateThreadMutation",
            json!({
                "title": title,
                "content": content,
                "objectId": object_id,
                "userId": user_id,
                "subscribe": subscribe,
                "sendEmail": send_email,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_eq!(result.status, expected_response);
    }

    #[rstest]
    // positive cases:
    // negative cases:
    // valid payload except content is empty
    #[case(StatusCode::BAD_REQUEST, 17774, 1, "", true, false)]
    // valid payload except thread does not exist
    #[case(StatusCode::BAD_REQUEST, 3, 1, "This is content", true, false)]
    #[actix_rt::test]
    async fn create_comment(
        #[case] expected_response: StatusCode,
        #[case] thread_id: i32,
        #[case] user_id: i32,
        #[case] content: &str,
        #[case] subscribe: bool,
        #[case] send_email: bool,
    ) {
        let mut transaction = begin_transaction().await;

        let result = Message::new(
            "ThreadCreateCommentMutation",
            json!({
                "threadId": thread_id,
                "userId": user_id,
                "content": content,
                "subscribe": subscribe,
                "sendEmail": send_email,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_eq!(result.status, expected_response);
    }

    #[rstest]
    // positive cases:
    // valid payload with unchanged content
    #[case(StatusCode::OK, false, 2, 15469, "Bitte neu einsortieren :)")]
    // valid payload with changed content
    #[case(StatusCode::OK, true, 2, 15469, "This is new content.")]
    // negative cases:
    // valid payload except content is empty
    #[case(StatusCode::BAD_REQUEST, false, 2, 15469, "")]
    // valid payload except user is not author of the comment
    #[case(StatusCode::BAD_REQUEST, false, 1, 15469, "This is new content.")]
    // valid payload except comment is trashed
    #[case(StatusCode::BAD_REQUEST, false, 1, 15468, "This is new content.")]
    // valid payload except comment is archived
    #[case(StatusCode::BAD_REQUEST, false, 10, 16740, "This is new content.")]
    #[actix_rt::test]
    async fn edit_comment(
        #[case] expected_response: StatusCode,
        #[case] content_should_change: bool,
        #[case] user_id: i32,
        #[case] comment_id: i32,
        #[case] content: &str,
    ) {
        let mut transaction = begin_transaction().await;

        let original_comment = &Message::new("UuidQuery", json!({ "id": comment_id, }))
            .execute_on(&mut transaction)
            .await
            .get_json();

        let result = Message::new(
            "EditCommentMutation",
            json!({
                "userId": user_id,
                "commentId": comment_id,
                "content": content,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_eq!(result.status, expected_response);
        match expected_response {
            StatusCode::OK => assert_eq!(result.get_json(), json!({ "success": true })),
            _ => {
                assert_json_include!(
                    actual: result.get_json(),
                    expected: json!({ "success": false })
                )
            }
        }

        Message::new("UuidQuery", json!({ "id": comment_id, }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|comment| {
                if content_should_change {
                    assert_ne!(comment["content"], original_comment["content"]);
                    // assert_ne!(comment["edit_date"], original_comment["edit_date"]);
                } else {
                    assert_eq!(comment["content"], original_comment["content"]);
                    // assert_eq!(comment["edit_date"], original_comment["edit_date"]);
                }
            });
    }
}
