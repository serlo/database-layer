#[cfg(test)]
mod set_name_and_description_mutation {
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
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_new_taxonomy_term() {
        for taxonomy_type in ALLOWED_TAXONOMY_TYPES_CREATE.iter() {
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

#[cfg(test)]
mod create_entity_link_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_entity_link() {
        let mut transaction = begin_transaction().await;

        let children_ids = [2059, 2327];
        let taxonomy_term_id = 1288;

        Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        for child_id in children_ids.iter() {
            let query_response = Message::new("UuidQuery", json!({ "id": child_id }))
                .execute_on(&mut transaction)
                .await;

            assert_ok_with(query_response, |result| {
                assert!(result["taxonomyTermIds"]
                    .as_array()
                    .unwrap()
                    .contains(&to_value(taxonomy_term_id).unwrap()));
            })
            .await;

            let events_response =
                Message::new("EventsQuery", json ! ({ "first": 1, "objectId": child_id }))
                    .execute_on(&mut transaction)
                    .await;

            assert_ok_with(events_response, |result| {
                assert_json_include ! (
                    actual: &result["events"][0],
                    expected: json ! ({
                        "__typename": "CreateTaxonomyLinkNotificationEvent",
                        "instance": "de",
                        "actorId": 1,
                        "objectId": taxonomy_term_id,
                        "parentId": taxonomy_term_id,
                        "childId": child_id
                    })
                );
            })
            .await;
        }
    }

    #[actix_rt::test]
    async fn fails_if_a_child_is_not_an_entity() {
        let mut transaction = begin_transaction().await;

        let children_ids = [2059, 1];
        let taxonomy_term_id = 1288;

        let response = Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "Entity with id 1 does not exist").await;
    }

    #[actix_rt::test]
    async fn fails_if_parent_is_not_a_taxonomy_term() {
        let mut transaction = begin_transaction().await;

        let children_ids = [2059, 2327];
        let taxonomy_term_id = 1;

        let response = Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "Taxonomy term with id 1 does not exist").await;
    }

    #[actix_rt::test]
    async fn fails_if_parent_and_child_are_in_different_instances() {
        let mut transaction = begin_transaction().await;

        let children_ids = [2059, 28952];
        let taxonomy_term_id = 7;

        let response = Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(
            response,
            "Entity 28952 and taxonomy term 7 are not in the same instance",
        )
        .await;
    }

    #[actix_rt::test]
    async fn does_not_store_same_link_twice() {
        let mut transaction = begin_transaction().await;

        let children_ids = [2059];
        let taxonomy_term_id = 1307;

        let count =
            count_taxonomy_entity_links(children_ids[0], taxonomy_term_id, &mut transaction).await;
        assert_eq!(count, 1);

        let response = Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok(response, json!({ "success": true })).await;

        let count =
            count_taxonomy_entity_links(children_ids[0], taxonomy_term_id, &mut transaction).await;
        assert_eq!(count, 1)
    }
}

#[cfg(test)]
mod delete_entity_links_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn deletes_entity_links() {
        let mut transaction = begin_transaction().await;

        let children_ids = [1949, 1543];
        let taxonomy_term_id = 24370;

        Message::new(
            "TaxonomyDeleteEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        for child_id in children_ids.iter() {
            let query_response = Message::new("UuidQuery", json!({ "id": child_id }))
                .execute_on(&mut transaction)
                .await;

            assert_ok_with(query_response, |result| {
                assert!(!result["taxonomyTermIds"]
                    .as_array()
                    .unwrap()
                    .contains(&to_value(taxonomy_term_id).unwrap()));
            })
            .await;

            let events_response =
                Message::new("EventsQuery", json ! ({ "first": 1, "objectId": child_id }))
                    .execute_on(&mut transaction)
                    .await;

            assert_ok_with(events_response, |result| {
                assert_json_include ! (
                    actual: &result["events"][0],
                    expected: json ! ({
                        "__typename": "RemoveTaxonomyLinkNotificationEvent",
                        "instance": "de",
                        "actorId": 1,
                        "objectId": taxonomy_term_id,
                        "parentId": taxonomy_term_id,
                        "childId": child_id
                    })
                );
            })
            .await;
        }
    }

    #[actix_rt::test]
    async fn fails_if_there_is_no_link_yet() {
        let mut transaction = begin_transaction().await;

        let children_ids = [1743, 2059];
        let taxonomy_term_id = 24503;

        let response = Message::new(
            "TaxonomyDeleteEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "Id 2059 is not linked to taxonomy term 24503").await;
    }

    #[actix_rt::test]
    async fn fails_if_it_would_leave_child_orphan() {
        let mut transaction = begin_transaction().await;

        let children_ids = [12957];
        let taxonomy_term_id = 1463;

        let response = Message::new(
            "TaxonomyDeleteEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(
            response,
            "Entity with id 12957 has to be linked to at least one taxonomy",
        )
        .await;
    }
}
