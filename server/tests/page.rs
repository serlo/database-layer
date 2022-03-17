#[cfg(test)]
mod add_revision_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn adds_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "PageAddRevisionMutation",
            json!({
                "pageId": 16256,
                "content": "test content",
                "title": "test title",
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({ "id": get_json(mutation_response).await["revisionId"] }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["content"], "test content");
            assert_eq!(result["title"], "test title");
            assert_eq!(result["authorId"], 1 as i32);
        })
        .await;
    }
}

mod create_mutation {
    use serde_json::Value::Null;
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_page() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "PageCreateMutation",
            json!({
                "content": "test content",
                "discussionsEnabled": false,
                "forumId": Null,
                "instance": "de",
                "licenseId": 1 as i32,
                "title": "test title",
                "userId": 1 as i32,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(response, |result| {
            assert_eq!(result["instance"], "de");
            assert_eq!(result["licenseId"], 1 as i32);
        })
        .await
    }
}
