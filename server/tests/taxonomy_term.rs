#[cfg(test)]
mod set_name_and_description_mutation {
    use assert_json_diff::assert_json_include;
    use test_utils::*;

    #[actix_rt::test]
    async fn sets_name_and_description() {
        for description in [Some("a description"), None] {
            let mut transaction = begin_transaction().await;

            let response = Message::new(
                "TaxonomyTermSetNameAndDescriptionMutation",
                json!({
                    "id": 7,
                    "userId": 1,
                    "name": "a name",
                    "description": description
                }),
            )
            .execute_on(&mut transaction)
            .await;

            assert_ok(response, json!({ "success": true })).await;

            let query_response = Message::new("UuidQuery", json!({ "id": 7 }))
                .execute_on(&mut transaction)
                .await;

            assert_ok_with(query_response, |result| {
                assert_eq!(result["name"], "a name");
                assert_eq!(result["description"].as_str(), description);
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

    #[actix_rt::test]
    async fn fails_when_taxonomy_term_does_not_exist() {
        let response = Message::new(
            "TaxonomyTermSetNameAndDescriptionMutation",
            json!({
                "id": 1,
                "userId": 1,
                "name": "a name",
                "description": "a description"
            }),
        )
        .execute()
        .await;

        assert_bad_request(response, "Taxonomy term with id 1 does not exist").await;
    }
}
