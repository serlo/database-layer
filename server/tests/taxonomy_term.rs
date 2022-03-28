#[cfg(test)]
mod set_name_and_description_mutation {
    use assert_json_diff::assert_json_include;
    use test_utils::*;

    #[actix_rt::test]
    async fn sets_name_and_description() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
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

        assert_ok(response, json!({ "success": true })).await;

        let query_response = Message::new("UuidQuery", json!({"id": 7}))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["name"], "a name");
            assert_eq!(result["description"], "a description");
        })
        .await;

        let events_response = Message::new("EventsQuery", json!({ "first": 1, "objectId": 7 }))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(events_response, |result| {
            assert_json_include!(
                actual: &result["events"][0],
                expected: json!({
                    "__typename": "SetTaxonomyTermNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "objectId": 7,
                    "taxonomyTermId": 7,
                })
            );
        })
        .await;
    }
}
