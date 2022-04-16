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

#[cfg(test)]
mod move_mutation {
    use assert_json_diff::assert_json_include;
    use test_utils::*;

    #[actix_rt::test]
    async fn moves_to_new_parent() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1394, 1454], "destination": 5, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok(response, json!({ "success": true })).await;

        let first_query_response = Message::new("UuidQuery", json!({ "id": 1394 }))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(first_query_response, |result| {
            assert_eq!(result["parentId"], 5);
        })
        .await;

        let second_query_response = Message::new("UuidQuery", json!({ "id": 1454 }))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(second_query_response, |result| {
            assert_eq!(result["parentId"], 5);
        })
        .await;

        let events_response = Message::new("EventsQuery", json!({ "first": 1, "objectId": 1394 }))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(events_response, |result| {
            assert_json_include!(
                actual: &result["events"][0],
                expected: json!({
                    "__typename": "SetTaxonomyParentNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "objectId": 1394,
                    "childId": 1394,
                    "previousParentId": 1288,
                    "parentId": 5
                })
            );
        })
        .await;
    }

    #[actix_rt::test]
    async fn fails_when_parent_and_child_have_same_id() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1288], "destination": 1288, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "Child cannot have same id 1288 as destination").await;
    }

    #[actix_rt::test]
    async fn fails_when_parent_does_not_exist() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1288], "destination": 1, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "Taxonomy term with id 1 does not exist").await;
    }

    #[actix_rt::test]
    async fn fails_when_child_does_not_exist() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1], "destination": 1288, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "Taxonomy term with id 1 does not exist").await;
    }

    #[actix_rt::test]
    async fn fails_when_trying_to_move_root() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [3], "destination": 5, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "root taxonomy term 3 cannot be moved").await;
    }

    #[actix_rt::test]
    async fn fails_when_previous_and_new_parent_are_same() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1300], "destination": 1288, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(
            response,
            "Taxonomy term with id 1300 already child of parent 1288",
        )
        .await;
    }

    #[actix_rt::test]
    async fn fails_when_child_and_new_parent_are_in_different_instances() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1300], "destination": 23594, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(
            response,
            "Taxonomy term with id 1300 cannot be moved to another instance",
        )
        .await;
    }
}

#[cfg(test)]
mod create_mutation {
    use assert_json_diff::assert_json_include;
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_new_taxonomy_term() {
        for taxonomy_type in TAXONOMY_TYPES_WITHOUT_ROOT.iter() {
            for description in [Some("a description"), None] {
                let mut transaction = begin_transaction().await;

                let mutation_response = Message::new(
                    "TaxonomyTermCreateMutation",
                    json! ({
                    "parentId": 1394,
                    "name": "a name",
                    "description": description,
                    "userId": 1,
                    "taxonomyType": taxonomy_type
                    }),
                )
                .execute_on(&mut transaction)
                .await;

                let new_taxonomy_id = get_json(mutation_response).await["id"].clone();

                let query_response = Message::new("UuidQuery", json!({ "id": new_taxonomy_id }))
                    .execute_on(&mut transaction)
                    .await;

                assert_ok_with(query_response, |result| {
                    assert_eq!(result["name"], "a name");
                    assert_eq!(result["description"].as_str(), description);
                    assert_eq!(result["parentId"], 1394);
                    assert_eq!(
                        from_value_to_taxonomy_type(result["type"].clone()),
                        *taxonomy_type
                    );
                })
                .await;

                let events_response = Message::new(
                    "EventsQuery",
                    json ! ({ "first": 1, "objectId": new_taxonomy_id }),
                )
                .execute_on(&mut transaction)
                .await;

                assert_ok_with(events_response, |result| {
                    assert_json_include ! (
                        actual: &result["events"][0],
                        expected: json ! ({
                            "__typename": "CreateTaxonomyTermNotificationEvent",
                            "instance": "de",
                            "actorId": 1,
                            "objectId": new_taxonomy_id,
                            "taxonomyTermId": new_taxonomy_id,
                        })
                    );
                })
                .await;
            }
        }
    }
}
