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
            json!({ "id": get_json(mutation_response).await["pageRevisionId"] }),
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
