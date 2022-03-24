#[cfg(test)]
mod set_name_and_description_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn sets_name_and_description() {
        let mut transaction = begin_transaction().await;

        Message::new(
            "TaxonomyTermSetNameAndDescriptionMutation",
            json!({
                "id": 7,
                "userId": 1,
                "name": "a name",
                "description": "a description"
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new("UuidQuery", json!({"id": 7}))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["name"], "a name");
            assert_eq!(result["description"], "a description");
        })
        .await;
    }
}
