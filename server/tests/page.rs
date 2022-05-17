#[cfg(test)]
mod add_revision_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn adds_revision() {
        let mut transaction = begin_transaction().await;

        let new_revision_id = Message::new(
            "PageAddRevisionMutation",
            json!({
                "pageId": 16256,
                "content": "test content",
                "title": "test title",
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await
        .get_json()["revisionId"]
            .clone();

        Message::new("UuidQuery", json!({ "id": new_revision_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["content"], "test content");
                assert_eq!(result["title"], "test title");
                assert_eq!(result["authorId"], 1 as i32);
            });
    }
}

mod create_mutation {
    use serde_json::Value::Null;
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_page() {
        Message::new(
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
        .execute()
        .await
        .should_be_ok_with(|result| {
            assert_eq!(result["instance"], "de");
            assert_eq!(result["licenseId"], 1 as i32);
        })
    }
}
