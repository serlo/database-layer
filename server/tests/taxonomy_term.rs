mod set_name_and_description_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn sets_name_and_description() {
        for description in [Some("a description"), None] {
            let mut transaction = begin_transaction().await;

            Message::new(
                "TaxonomyTermSetNameAndDescriptionMutation",
                json!({
                    "id": 7,
                    "userId": 1,
                    "name": "a name",
                    "description": description
                }),
            )
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with_body(json!({ "success": true }));

            Message::new("UuidQuery", json!({ "id": 7 }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
                    assert_eq!(result["name"], "a name");
                    assert_eq!(result["description"].as_str(), description);
                });

            Message::new("EventsQuery", json!({ "first": 1, "objectId": 7 }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
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
                });
        }
    }

    #[actix_rt::test]
    async fn fails_when_taxonomy_term_does_not_exist() {
        Message::new(
            "TaxonomyTermSetNameAndDescriptionMutation",
            json!({
                "id": 1,
                "userId": 1,
                "name": "a name",
                "description": "a description"
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_with_bad_request_when_repeated_name_in_same_instance() {
        let mut transaction = begin_transaction().await;

        Message::new(
            "TaxonomyTermSetNameAndDescriptionMutation",
            json!({
                "id": 1292 as i32,
                "userId": 1 as i32,
                "name": "repeated_name",
                "description": "bla"
            }),
        )
        .execute_on(&mut transaction)
        .await;

        Message::new(
            "TaxonomyTermSetNameAndDescriptionMutation",
            json!({
                "id": 1293 as i32,
                "userId": 1 as i32,
                "name": "repeated_name",
                "description": "bla"
            }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_bad_request();
    }
}

mod move_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn moves_to_new_parent() {
        let mut transaction = begin_transaction().await;

        Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1394, 1454], "destination": 5, "userId": 1 }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "success": true }));

        Message::new("UuidQuery", json!({ "id": 1394 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["parentId"], 5);
            });

        Message::new("UuidQuery", json!({ "id": 1454 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["parentId"], 5);
            });

        Message::new("EventsQuery", json!({ "first": 1, "objectId": 1394 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
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
            });
    }

    #[actix_rt::test]
    async fn fails_when_parent_and_child_have_same_id() {
        Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1288], "destination": 1288, "userId": 1 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_parent_does_not_exist() {
        Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1288], "destination": 1, "userId": 1 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_child_does_not_exist() {
        Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1], "destination": 1288, "userId": 1 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_trying_to_move_root() {
        Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [3], "destination": 5, "userId": 1 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_previous_and_new_parent_are_same() {
        Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1300], "destination": 1288, "userId": 1 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_child_and_new_parent_are_in_different_instances() {
        Message::new(
            "TaxonomyTermMoveMutation",
            json!({ "childrenIds": [1300], "destination": 23594, "userId": 1 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

mod create_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_new_taxonomy_term() {
        for taxonomy_type in ALLOWED_TAXONOMY_TYPES_CREATE.iter() {
            for description in [Some("a description"), None] {
                let mut transaction = begin_transaction().await;

                let new_taxonomy_id = Message::new(
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
                .await
                .get_json()["id"]
                    .clone();

                Message::new("UuidQuery", json!({ "id": new_taxonomy_id }))
                    .execute_on(&mut transaction)
                    .await
                    .should_be_ok_with(|result| {
                        assert_eq!(result["name"], "a name");
                        assert_eq!(result["description"].as_str(), description);
                        assert_eq!(result["parentId"], 1394);
                        assert_eq!(
                            from_value_to_taxonomy_type(result["type"].clone()),
                            *taxonomy_type
                        );
                    });

                Message::new(
                    "EventsQuery",
                    json ! ({ "first": 1, "objectId": new_taxonomy_id }),
                )
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
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
                });
            }
        }
    }

    #[actix_rt::test]
    async fn fails_with_bad_request_if_parent_does_not_exist() {
        Message::new(
            "TaxonomyTermCreateMutation",
            json! ({
            "parentId": 1 as i32,
            "name": "a name",
            "description": "a description",
            "userId": 1 as i32,
            "taxonomyType": "topic"
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_with_bad_request_when_repeated_name_in_same_instance() {
        let mut transaction = begin_transaction().await;

        Message::new(
            "TaxonomyTermCreateMutation",
            json! ({
            "parentId": 1394 as i32,
            "name": "repeated_name",
            "description": "bla",
            "userId": 1 as i32,
            "taxonomyType": "topic-folder"
            }),
        )
        .execute_on(&mut transaction)
        .await;

        Message::new(
            "TaxonomyTermCreateMutation",
            json! ({
            "parentId": 1394 as i32,
            "name": "repeated_name",
            "description": "bla",
            "userId": 1 as i32,
            "taxonomyType": "topic-folder"
            }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_bad_request();
    }
}

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
            Message::new("UuidQuery", json!({ "id": child_id }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
                    assert!(result["taxonomyTermIds"]
                        .as_array()
                        .unwrap()
                        .contains(&to_value(taxonomy_term_id).unwrap()));
                });

            Message::new("EventsQuery", json ! ({ "first": 1, "objectId": child_id }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
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
                });
        }
    }

    #[actix_rt::test]
    async fn fails_if_a_child_is_not_an_entity() {
        Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": [2059, 1],
                "taxonomyTermId": 1288
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_if_a_child_cannot_be_linked_into_a_taxonomy_term() {
        Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({ "userId": 1, "entityIds": [29648], "taxonomyTermId": 1288 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_if_parent_is_not_a_taxonomy_term() {
        Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": [2059, 2327],
                "taxonomyTermId": 1
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_if_parent_and_child_are_in_different_instances() {
        Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": [2059, 28952],
                "taxonomyTermId": 7
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn does_not_store_same_link_twice() {
        let mut transaction = begin_transaction().await;

        let entity_id = 2059;
        let taxonomy_term_id = 1307;

        let count_before =
            count_taxonomy_entity_links(entity_id, taxonomy_term_id, &mut transaction).await;

        Message::new(
            "TaxonomyCreateEntityLinksMutation",
            json! ({
                "userId": 1,
                "entityIds": [entity_id],
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute()
        .await
        .should_be_ok_with_body(json!({ "success": true }));

        assert_eq!(
            count_before,
            count_taxonomy_entity_links(entity_id, taxonomy_term_id, &mut transaction).await
        );
    }
}

mod delete_entity_links_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn deletes_entity_links() {
        let mut transaction = begin_transaction().await;

        let children_ids = [1949, 1543];
        let taxonomy_term_id = 24370;

        Message::new(
            "TaxonomyDeleteEntityLinksMutation",
            json! ({ "userId": 1, "entityIds": children_ids, "taxonomyTermId": taxonomy_term_id }),
        )
        .execute_on(&mut transaction)
        .await;

        for child_id in children_ids.iter() {
            Message::new("UuidQuery", json!({ "id": child_id }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
                    assert!(!result["taxonomyTermIds"]
                        .as_array()
                        .unwrap()
                        .contains(&to_value(taxonomy_term_id).unwrap()));
                });

            Message::new("EventsQuery", json ! ({ "first": 1, "objectId": child_id }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
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
                });
        }
    }

    #[actix_rt::test]
    async fn fails_if_there_is_no_link_yet() {
        Message::new(
            "TaxonomyDeleteEntityLinksMutation",
            json! ({ "userId": 1, "entityIds": [1743, 2059], "taxonomyTermId": 24503 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_if_it_would_leave_child_orphan() {
        Message::new(
            "TaxonomyDeleteEntityLinksMutation",
            json! ({ "userId": 1, "entityIds": [12957], "taxonomyTermId": 1463 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

mod sort_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn sorts_children() {
        let mut transaction = begin_transaction().await;

        let children_ids = [2021, 1949, 24390, 1455];
        let taxonomy_term_id = 24389;

        Message::new(
            "TaxonomySortMutation",
            json! ({
                "userId": 1,
                "childrenIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        Message::new("UuidQuery", json!({ "id": taxonomy_term_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["childrenIds"], to_value(children_ids).unwrap());
            });

        Message::new("EventsQuery", json!({ "first": 1, "objectId": 3 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_json_include!(
                    actual: &result["events"][0],
                    expected: json!({
                        "__typename": "SetTaxonomyTermNotificationEvent",
                        "instance": "de",
                        "actorId": 1,
                        "objectId": 3
                    })
                );
            });
    }

    #[actix_rt::test]
    async fn is_ok_when_children_order_is_same_not_triggering_event() {
        let mut transaction = begin_transaction().await;

        let children_ids = [1557, 1553, 2107, 24398, 30560];
        let taxonomy_term_id = 1338;

        Message::new(
            "TaxonomySortMutation",
            json! ({
                "userId": 1,
                "childrenIds": children_ids,
                "taxonomyTermId": taxonomy_term_id
            }),
        )
        .execute_on(&mut transaction)
        .await;

        Message::new("UuidQuery", json!({ "id": taxonomy_term_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["childrenIds"], to_value(children_ids).unwrap());
            });

        Message::new("EventsQuery", json!({ "first": 1, "objectId": 3 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_json_include!(
                    actual: &result["events"][0],
                    expected: json!({
                        // that means another event
                        "__typename": "SetTaxonomyParentNotificationEvent",
                    })
                );
            });
    }

    #[actix_rt::test]
    async fn fails_with_bad_request_if_taxonomy_does_not_exist() {
        Message::new(
            "TaxonomySortMutation",
            json! ({
                "userId": 1,
                "childrenIds": [2021, 1949, 24390, 1455],
                "taxonomyTermId": 1
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_if_the_children_ids_do_not_match_the_linked_entities_ids() {
        Message::new(
            "TaxonomySortMutation",
            json! ({
                "userId": 1,
                "childrenIds": [1743, 2059],
                "taxonomyTermId": 24503 as i32
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}
