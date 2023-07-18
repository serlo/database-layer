mod unrevised_entities_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_list_of_unrevised_entities() {
        Message::new("UnrevisedEntitiesQuery", json!({}))
            .execute()
            .await
            .should_be_ok_with_body(
                json!({ "unrevisedEntityIds": [26892, 33582, 34741, 34907, 35247, 35556] }),
            );
    }
}

mod add_revision_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn adds_revision() {
        for revision in EntityTestWrapper::all().iter() {
            let mut transaction = begin_transaction().await;

            let revision_id = Message::new(
                "EntityAddRevisionMutation",
                json!({
                    "revisionType": revision.revision_type,
                    "input": {
                        "changes": "test changes",
                        "entityId": revision.entity_id,
                        "needsReview": true,
                        "subscribeThis": false,
                        "subscribeThisByEmail": false,
                        "fields": revision.fields()
                    },
                    "userId": 1,
                }),
            )
            .execute_on(&mut transaction)
            .await
            .get_json()["revisionId"]
                .clone();

            Message::new("UuidQuery", json!({ "id": revision_id }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
                    assert_eq!(result["changes"], "test changes");
                    if revision.query_fields.is_some() {
                        for (key, value) in revision.query_fields.clone().unwrap() {
                            assert_eq!(result[key], value);
                        }
                    } else {
                        for (key, value) in revision.fields() {
                            assert_eq!(result[key], value);
                        }
                    }
                });

            assert_event_revision_ok(revision_id, revision.entity_id, &mut transaction).await;
        }
    }

    #[actix_rt::test]
    async fn does_not_add_revision_if_fields_are_same() {
        for revision in EntityTestWrapper::all().iter() {
            let mut transaction = begin_transaction().await;

            let first_revision_id = Message::new(
                "EntityAddRevisionMutation",
                json!({
                    "revisionType": revision.revision_type,
                    "input": {
                        "changes": "test changes",
                        "entityId": revision.entity_id,
                        "needsReview": true,
                        "subscribeThis": false,
                        "subscribeThisByEmail": false,
                        "fields": revision.fields()
                    },
                    "userId": 1
                }),
            )
            .execute_on(&mut transaction)
            .await
            .get_json()["revisionId"]
                .clone();
            let first_revision_ids = get_revisions(revision.entity_id, &mut transaction).await;

            let second_revision_id = Message::new(
                "EntityAddRevisionMutation",
                json!({
                    "revisionType": revision.revision_type,
                    "input": {
                        "changes": "second edit",
                        "entityId": revision.entity_id,
                        "needsReview": true,
                        "subscribeThis": false,
                        "subscribeThisByEmail": false,
                        "fields": revision.fields()
                    },
                    "userId": 1
                }),
            )
            .execute_on(&mut transaction)
            .await
            .get_json()["revisionId"]
                .clone();
            let second_revision_ids = get_revisions(revision.entity_id, &mut transaction).await;

            assert_eq!(first_revision_id, second_revision_id);
            assert_eq!(first_revision_ids, second_revision_ids);
        }
    }

    #[actix_rt::test]
    async fn does_not_add_revision_if_cohesive_does_not_exist() {
        let exercise_group_revision = Message::new("UuidQuery", json!({ "id": 2218 }))
            .execute()
            .await
            .get_json();

        let new_revision = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": "ExerciseGroupRevision",
                "input": {
                    "changes": "second edit",
                    "entityId": exercise_group_revision["repositoryId"],
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "cohesive": "false",
                        "content": exercise_group_revision["content"]
                    }
                },
                "userId": 1
            }),
        )
        .execute()
        .await
        .get_json();

        assert_eq!(exercise_group_revision["id"], new_revision["revisionId"]);
    }

    #[actix_rt::test]
    async fn does_not_add_revision_if_course_page_has_icon() {
        let revision = Message::new("UuidQuery", json!({ "id": 31315 }))
            .execute()
            .await
            .get_json();

        let new_revision = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": "CoursePageRevision",
                "input": {
                    "changes": "second edit",
                    "entityId": revision["repositoryId"],
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "title": revision["title"],
                        "content": revision["content"]
                    }
                },
                "userId": 1
            }),
        )
        .execute()
        .await
        .get_json();

        assert_eq!(revision["id"], new_revision["revisionId"]);
    }

    async fn get_revisions(id: i32, transaction: &mut sqlx::Transaction<'_, sqlx::MySql>) -> Value {
        Message::new("UuidQuery", json!({ "id": id }))
            .execute_on(transaction)
            .await
            .get_json()["revisionIds"]
            .clone()
    }
}

mod create_mutation {
    use rstest::*;
    use server::uuid::EntityType;
    use test_utils::{assert_eq, *};

    #[rstest]
    #[case(EntityType::Applet, Some(7), Option::<i32>::None)]
    #[case(EntityType::Article, Some(7), Option::<i32>::None)]
    #[case(EntityType::Course, Some(7), Option::<i32>::None)]
    #[case(EntityType::CoursePage, Option::<i32>::None, Some(18514))]
    #[case(EntityType::Event, Some(7), Option::<i32>::None)]
    #[case(EntityType::Exercise, Some(7), Option::<i32>::None)]
    #[case(EntityType::ExerciseGroup, Some(7), Option::<i32>::None)]
    #[case(EntityType::GroupedExercise, Option::<i32>::None, Some(2217))]
    #[case(EntityType::Solution, Option::<i32>::None, Some(4827))]
    #[case(EntityType::Video, Some(7), Option::<i32>::None)]
    #[actix_rt::test]
    async fn creates_entity(
        #[case] entity_type: EntityType,
        #[case] taxonomy_term_id: Option<i32>,
        #[case] parent_id: Option<i32>,
    ) {
        let mut transaction = begin_transaction().await;

        let new_entity_id = Message::new(
            "EntityCreateMutation",
            json!({
               "entityType": entity_type,
               "input": {
                   "changes": "test changes",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": taxonomy_term_id,
                    "parentId": parent_id,
                    "needsReview": false,
                    "fields": std::collections::HashMap::from([
                        ("content", "I am a new exercise!"),
                        ("description", "test description"),
                        ("metaDescription", "test metaDescription"),
                        ("metaTitle", "test metaTitle"),
                        ("title", "test title"),
                        ("url", "test url"),
                        ("cohesive", "true"),
                   ]),
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await
        .get_json()["id"]
            .clone();

        Message::new("UuidQuery", json!({ "id": new_entity_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(
                    from_value_to_entity_type(result["__typename"].clone()),
                    entity_type
                );
                assert_eq!(result["licenseId"], 1_i32);
                assert_eq!(result["instance"], "de");
            });

        Message::new(
            "EventsQuery",
            json!({ "first": 4, "objectId": new_entity_id }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with(|result| {
            let (parent_event_name, parent_id, object_id) = match taxonomy_term_id {
                Some(taxonomy_term_id) => (
                    "CreateTaxonomyLinkNotificationEvent",
                    taxonomy_term_id,
                    taxonomy_term_id,
                ),
                None => (
                    "CreateEntityLinkNotificationEvent",
                    parent_id.unwrap(),
                    new_entity_id.as_i64().unwrap() as i32,
                ),
            };
            assert_json_include!(
                actual: &result["events"][0],
                expected: json!({
                    "__typename": "CheckoutRevisionNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "repositoryId": new_entity_id
                })
            );
            assert_json_include!(
                actual: &result["events"][1],
                expected: json!({
                    "__typename": "CreateEntityRevisionNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "entityId": new_entity_id
                })
            );
            assert_json_include!(
                actual: &result["events"][2],
                expected: json!({
                    "__typename": "CreateEntityNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "entityId": new_entity_id
                })
            );
            assert_json_include!(
                actual: &result["events"][3],
                expected: json!({
                    "__typename": parent_event_name,
                    "instance": "de",
                    "actorId": 1,
                    "objectId": object_id,
                    "parentId": parent_id,
                    "childId": new_entity_id
                })
            );
        });
    }

    #[fixture]
    async fn exercise_with_solution<'a>() -> (sqlx::Transaction<'a, sqlx::MySql>, Value, Value) {
        let mut transaction = begin_transaction().await;

        let id_new_exercise = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": EntityType::Exercise,
                "input": {
                    "changes": "Creating a new exercise",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": Some(7),
                    "parentId": Option::<i32>::None,
                    "needsReview": false,
                    "fields": std::collections::HashMap::from([
                        ("content", "I am a new exercise!"),
                        ("description", "test description"),
                        ("metaDescription", "test metaDescription"),
                        ("metaTitle", "test metaTitle"),
                        ("title", "test title"),
                        ("url", "test url"),
                        ("cohesive", "true"),
                    ]),
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await
        .get_json()["id"]
            .clone();

        let id_solution = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": EntityType::Solution,
                "input": {
                    "changes": "Creating a solution",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": Option::<i32>::None,
                    "parentId": id_new_exercise,
                    "needsReview": false,
                    "fields": std::collections::HashMap::from([
                        ("content", "I am a new solution!"),
                        ("description", "test description"),
                        ("metaDescription", "test metaDescription"),
                        ("metaTitle", "test metaTitle"),
                        ("title", "test title"),
                        ("url", "test url"),
                        ("cohesive", "true"),
                    ]),
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await
        .get_json()["id"]
            .clone();

        (transaction, id_new_exercise, id_solution)
    }

    #[rstest]
    #[actix_rt::test]
    async fn creates_no_second_solution_for_same_exercise<'a>(
        #[future] exercise_with_solution: (sqlx::Transaction<'a, sqlx::MySql>, Value, Value),
    ) {
        let (mut transaction, id_new_exercise, _id_first_solution) = exercise_with_solution.await;

        Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": EntityType::Solution,
                "input": {
                    "changes": "Creating another solution for the same exercise",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": Option::<i32>::None,
                    "parentId": id_new_exercise,
                    "needsReview": false,
                    "fields": std::collections::HashMap::from([
                        ("content", "I am another solution!"),
                        ("description", "test description"),
                        ("metaDescription", "test metaDescription"),
                        ("metaTitle", "test metaTitle"),
                        ("title", "test title"),
                        ("url", "test url"),
                        ("cohesive", "true"),
                    ]),
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_bad_request();
    }

    #[rstest]
    #[actix_rt::test]
    async fn creates_new_solution_for_exercise_if_existing_solution_is_trashed<'a>(
        #[future] exercise_with_solution: (sqlx::Transaction<'a, sqlx::MySql>, Value, Value),
    ) {
        let (mut transaction, id_new_exercise, id_first_solution) = exercise_with_solution.await;

        Message::new(
            "UuidSetStateMutation",
            json!({
                "ids": [id_first_solution],
                "userId": 1,
                "trashed": true
            }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();

        Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": EntityType::Solution,
                "input": {
                    "changes": "Creating another solution for the same exercise",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": Option::<i32>::None,
                    "parentId": id_new_exercise,
                    "needsReview": false,
                    "fields": std::collections::HashMap::from([
                        ("content", "I am another solution!"),
                        ("description", "test description"),
                        ("metaDescription", "test metaDescription"),
                        ("metaTitle", "test metaTitle"),
                        ("title", "test title"),
                        ("url", "test url"),
                        ("cohesive", "true"),
                    ]),
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();
    }

    #[rstest]
    #[actix_rt::test]
    async fn trashed_solution_is_not_returned_with_exercise<'a>(
        #[future] exercise_with_solution: (sqlx::Transaction<'a, sqlx::MySql>, Value, Value),
    ) {
        let (mut transaction, id_exercise, id_solution) = exercise_with_solution.await;

        Message::new(
            "UuidSetStateMutation",
            json!({
                "ids": [id_solution],
                "userId": 1,
                "trashed": true
            }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();

        Message::new("UuidQuery", json!({ "id": id_exercise }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(
                    result["solutionIds"],
                    to_value(serde_json::Value::Null).unwrap()
                );
            });
    }

    #[rstest]
    #[actix_rt::test]
    async fn non_trashed_solution_is_returned_with_exercise<'a>(
        #[future] exercise_with_solution: (sqlx::Transaction<'a, sqlx::MySql>, Value, Value),
    ) {
        let (mut transaction, id_exercise, id_solution) = exercise_with_solution.await;

        Message::new("UuidQuery", json!({ "id": id_exercise }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(
                    result["solutionIds"],
                    to_value(id_solution.as_array()).unwrap()
                );
            });
    }

    #[actix_rt::test]
    async fn puts_newly_created_entity_as_last_sibling() {
        for entity in EntityTestWrapper::all().iter() {
            let mut transaction = begin_transaction().await;

            let new_entity_id = Message::new(
                "EntityCreateMutation",
                json!({
                    "entityType": entity.typename,
                    "input": {
                        "changes": "test changes",
                        "subscribeThis": false,
                        "subscribeThisByEmail": false,
                        "licenseId": 1,
                        "taxonomyTermId": entity.taxonomy_term_id,
                        "parentId": entity.parent_id,
                        "needsReview": true,
                        "fields": entity.fields(),
                    },
                    "userId": 1,
                }),
            )
            .execute_on(&mut transaction)
            .await
            .get_json()["id"]
                .clone();

            let parent_element_id = entity.taxonomy_term_id.or(entity.parent_id).unwrap();

            let children_ids_name = match entity.typename {
                EntityType::CoursePage => "pageIds",
                EntityType::GroupedExercise => "exerciseIds",
                // The parent of solution, exercise group, doesn't have an array of solutions, just one
                EntityType::Solution => continue,
                _ => "childrenIds",
            };

            Message::new("UuidQuery", json!({ "id": parent_element_id }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
                    let children_ids_value = result[children_ids_name].clone();
                    let children_ids = children_ids_value.as_array().unwrap();
                    assert_eq!(children_ids[children_ids.len() - 1], new_entity_id);
                });
        }
    }

    #[actix_rt::test]
    async fn checkouts_new_revision_when_needs_review_is_true() {
        Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Article",
                "input": {
                    "changes": "test changes",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": false,
                    "fields": {
                        "content": "content",
                        "title": "title",
                        "metaTitle": "metaTitle",
                        "metaDescription": "metaDescription"
                    },
                },
                "userId": 1,
            }),
        )
        .execute()
        .await
        .should_be_ok_with(|result| assert!(!result["currentRevisionId"].is_null()));
    }

    #[actix_rt::test]
    async fn fails_when_parent_is_no_entity() {
        Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Solution",
                "input": {
                    "changes": "test changes",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "parentId": 1,
                    "needsReview": true,
                    "fields": {
                        "content": "content",
                    },
                },
                "userId": 1_i32,
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_taxonomy_term_does_not_exist() {
        Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Article",
                "input": {
                    "changes": "test changes",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 1,
                    "needsReview": true,
                    "fields": {
                        "content": "content",
                        "title": "title",
                        "metaTitle": "metaTitle",
                        "metaDescription": "metaDescription"
                    },
                },
                "userId": 1_i32,
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

mod deleted_entities_query {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn gives_back_first_deleted_entities() {
        let first: usize = 3;

        Message::new("DeletedEntitiesQuery", json!({ "first": first }))
            .execute()
            .await
            .should_be_ok_with_body(json!({
              "success": true,
              "deletedEntities": [
                {
                  "dateOfDeletion": "2022-06-03T18:13:35+02:00",
                  "id": 28952
                },
                {
                  "dateOfDeletion": "2015-02-25T10:36:41+01:00",
                  "id": 35147
                },
                {
                  "dateOfDeletion": "2015-02-23T18:00:14+01:00",
                  "id": 33028
                }
              ]
            }));
    }

    #[actix_rt::test]
    async fn gives_back_first_deleted_entities_after_date() {
        Message::new(
            "DeletedEntitiesQuery",
            json!({ "first": 3, "after": "2015-02-22T14:57:14+01:00" }),
        )
        .execute()
        .await
        .should_be_ok_with(|result| {
            assert_eq!(
                result["deletedEntities"],
                json!([
                    {
                      "dateOfDeletion": "2015-02-22T14:56:59+01:00",
                      "id": 34721
                    },
                    {
                      "dateOfDeletion": "2015-02-22T14:56:18+01:00",
                      "id": 34716
                    },
                    {
                      "dateOfDeletion": "2015-02-19T15:32:34+01:00",
                      "id": 34717
                    }
                  ]
                )
            );
        });
    }

    #[actix_rt::test]
    async fn gives_back_first_deleted_entities_of_instance() {
        let mut transaction = begin_transaction().await;

        Message::new(
            "DeletedEntitiesQuery",
            json!({ "first": 1, "instance": "en" }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({
          "success": true,
          "deletedEntities": [
            {
              "dateOfDeletion": "2022-06-03T18:13:35+02:00",
              "id": 28952
            }
          ]
        }
        ));
    }

    #[actix_rt::test]
    async fn fails_when_date_format_is_wrong() {
        Message::new(
            "DeletedEntitiesQuery",
            json!({ "first": 4, "after": "no date" }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

mod set_license_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn sets_license_and_creates_new_event() {
        let mut transaction = begin_transaction().await;

        let user_id: i32 = 1;
        let entity_id: i32 = 1495;
        let license_id: i32 = 2;

        Message::new(
            "EntitySetLicenseMutation",
            json!({"userId": user_id, "entityId": entity_id, "licenseId": license_id}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "success": true, }));

        Message::new("UuidQuery", json!({ "id": entity_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| assert_eq!(result["licenseId"], license_id));

        Message::new(
            "EventsQuery",
            json!({ "first": 1_usize, "objectId": entity_id as usize}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with(|result| {
            assert_json_include!(
                actual: &result["events"][0],
                expected: json!({
                    "__typename": "SetLicenseNotificationEvent",
                    "instance": "de",
                    "actorId": user_id,
                    "objectId": entity_id,
                })
            )
        });
    }

    #[actix_rt::test]
    async fn fails_when_entity_does_not_exist() {
        Message::new(
            "EntitySetLicenseMutation",
            json!({"userId": 1, "entityId": 0, "licenseId": 2}),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_with_bad_request_when_user_does_not_exist() {
        Message::new(
            "EntitySetLicenseMutation",
            json!({"userId": 0, "entityId": 1495, "licenseId": 2}),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn does_not_set_a_new_event_log_entry_for_same_license_id() {
        let mut transaction = begin_transaction().await;

        let entity_id: i32 = 1495;

        Message::new(
            "EntitySetLicenseMutation",
            json!({"userId": 1, "entityId": entity_id, "licenseId": 1}),
        )
        .execute_on(&mut transaction)
        .await;

        Message::new(
            "EventsQuery",
            json!({ "first": 1_usize, "objectId": entity_id as usize}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with(|result| {
            assert_json_include!(
                actual: &result["events"][0],
                expected: json!({
                    "__typename": "CreateTaxonomyLinkNotificationEvent",
                })
            )
        });
    }
}

mod sort_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn sorts_children() {
        let mut transaction = begin_transaction().await;

        let children_ids = [9911, 9919, 2233, 2225, 5075, 9899, 9907];
        let entity_id = 2223;

        Message::new(
            "EntitySortMutation",
            json!({ "childrenIds": children_ids, "entityId": entity_id }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "success": true }));

        Message::new("UuidQuery", json!({ "id": entity_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["exerciseIds"], to_value(children_ids).unwrap());
            });
    }

    #[actix_rt::test]
    async fn sorts_children_when_also_subset_is_send() {
        let mut transaction = begin_transaction().await;

        let children_ids = [9911, 2233, 5075, 9907];
        let entity_id = 2223;

        Message::new(
            "EntitySortMutation",
            json!({ "childrenIds": children_ids, "entityId": entity_id }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "success": true }));

        Message::new("UuidQuery", json!({ "id": entity_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(
                    result["exerciseIds"],
                    to_value([9911, 2233, 5075, 9907, 2225, 9899, 9919]).unwrap()
                );
            });
    }

    #[actix_rt::test]
    async fn fails_with_bad_request_if_entity_id_does_not_belong_to_an_entity() {
        Message::new(
            "EntitySortMutation",
            json! ({ "childrenIds": [], "entityId": 1 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_if_the_children_ids_are_not_children_of_the_entity_id() {
        Message::new(
            "EntitySortMutation",
            json! ({ "childrenIds": [9911, 2059], "entityId": 2223 }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}
