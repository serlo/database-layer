#[cfg(test)]
mod unrevised_entities_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_list_of_unrevised_entities() {
        let r = Message::new("UnrevisedEntitiesQuery", json!({}))
            .execute()
            .await;

        assert_ok(
            r,
            json!({ "unrevisedEntityIds": [26892, 33582, 34741, 34907, 35247, 35556] }),
        )
        .await;
    }
}

#[cfg(test)]
mod add_revision_mutation {
    use server::uuid::abstract_entity_revision::EntityRevisionType;
    use test_utils::*;

    #[actix_rt::test]
    async fn adds_applet_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::Applet,
                "input": {
                    "changes": "test changes",
                    "entityId": 35596,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                        "metaDescription": "test metaDescription",
                        "metaTitle": "test metaTitle",
                        "title": "test title",
                        "url": "test url",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
            assert_eq!(result["metaTitle"], "test metaTitle");
            assert_eq!(result["metaDescription"], "test metaDescription");
            assert_eq!(result["title"], "test title");
            assert_eq!(result["url"], "test url");
        })
        .await;
    }

    #[actix_rt::test]
    async fn adds_article_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::Article,
                "input": {
                    "changes": "test changes",
                    "entityId": 1503,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                        "metaDescription": "test metaDescription",
                        "metaTitle": "test metaTitle",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
            assert_eq!(result["metaTitle"], "test metaTitle");
            assert_eq!(result["metaDescription"], "test metaDescription");
            assert_eq!(result["title"], "test title");
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_course_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::Course,
                "input": {
                    "changes": "test changes",
                    "entityId": 18275,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "description": "test description",
                        "metaDescription": "test metaDescription",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test description");
            assert_eq!(result["metaDescription"], "test metaDescription");
            assert_eq!(result["title"], "test title");
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_course_page_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::CoursePage,
                "input": {
                    "changes": "test changes",
                    "entityId": 18521,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
            assert_eq!(result["title"], "test title");
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_event_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::Event,
                "input": {
                    "changes": "test changes",
                    "entityId": 35554,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                        "metaDescription": "test metaDescription",
                        "metaTitle": "test metaTitle",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
            assert_eq!(result["metaTitle"], "test metaTitle");
            assert_eq!(result["metaDescription"], "test metaDescription");
            assert_eq!(result["title"], "test title");
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_exercise_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::Exercise,
                "input": {
                    "changes": "test changes",
                    "entityId": 2327,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_exercise_group_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::ExerciseGroup,
                "input": {
                    "changes": "test changes",
                    "entityId": 2217,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                        "cohesive": "true",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
            assert_eq!(result["cohesive"], true);
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_grouped_exercise_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::GroupedExercise,
                "input": {
                    "changes": "test changes",
                    "entityId": 2219,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_solution_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::Solution,
                "input": {
                    "changes": "test changes",
                    "entityId": 2221,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["content"], "test content");
        })
        .await;
    }
    #[actix_rt::test]
    async fn adds_video_revision() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityAddRevisionMutation",
            json!({
                "revisionType": EntityRevisionType::Video,
                "input": {
                    "changes": "test changes",
                    "entityId": 16078,
                    "needsReview": true,
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "fields": {
                        "content": "test content",
                        "description": "test description",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["revisionId"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["changes"], "test changes");
            assert_eq!(result["url"], "test content");
            assert_eq!(result["content"], "test description");
            assert_eq!(result["title"], "test title");
        })
        .await;
    }
}

#[cfg(test)]
mod create_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_applet() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Applet",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "content": "test content",
                        "metaDescription": "test metaDescription",
                        "metaTitle": "test metaTitle",
                        "title": "test title",
                        "url": "test url"
                    }
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "Applet");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_article() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Article",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "content": "test content",
                        "metaDescription": "test metaDescription",
                        "metaTitle": "test metaTitle",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "Article");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_course() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Course",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "description": "test description",
                        "metaDescription": "test metaDescription",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "Course");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_course_page() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "CoursePage",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "needsReview": true,
                    "parentId": 18514,
                    "fields": {
                        "content": "test content",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "CoursePage");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_event() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Event",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "content": "test content",
                        "metaDescription": "test metaDescription",
                        "metaTitle": "test metaTitle",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "Event");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_exercise() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Exercise",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "content": "test content",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "Exercise");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_exercise_group() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "ExerciseGroup",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "content": "test content",
                        "cohesive": "true",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "ExerciseGroup");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_grouped_exercise() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "GroupedExercise",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "needsReview": true,
                    "parentId": 2217,
                    "fields": {
                        "content": "test content",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "GroupedExercise");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_solution() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Solution",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "needsReview": true,
                    "parentId": 2219,
                    "fields": {
                        "content": "test content",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "Solution");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn creates_video() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Video",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "content": "test content",
                        "description": "test description",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let query_response = Message::new(
            "UuidQuery",
            json!({"id": get_json(mutation_response).await["id"]}),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["__typename"], "Video");
            assert_eq!(result["licenseId"], 1 as i32);
            assert_eq!(result["instance"], "de");
        })
        .await;
    }

    #[actix_rt::test]
    async fn triggers_events_with_param_taxonomy_term_id() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "Article",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "taxonomyTermId": 7,
                    "needsReview": true,
                    "fields": {
                        "content": "test content",
                        "metaDescription": "test metaDescription",
                        "metaTitle": "test metaTitle",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let new_entity_id = get_json(mutation_response).await["id"].clone();

        let events_response = Message::new(
            "EventsQuery",
            json!({ "first": 5, "objectId": new_entity_id }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok_with(events_response, |result| {
            assert_json_include!(
                actual: &result["events"][0],
                expected: json!({
                    "__typename": "CreateTaxonomyLinkNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "objectId": 7,
                    "parentId": 7,
                    "childId": new_entity_id
                })
            );
            assert_json_include!(
                actual: &result["events"][3],
                expected: json!({
                    "__typename": "CreateEntityNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "entityId": new_entity_id
                })
            );
        })
        .await;
    }

    #[actix_rt::test]
    async fn triggers_events_with_param_parent_id() {
        let mut transaction = begin_transaction().await;

        let mutation_response = Message::new(
            "EntityCreateMutation",
            json!({
                "entityType": "CoursePage",
                "input": {
                    "changes": "test changes",
                    "instance": "de",
                    "subscribeThis": false,
                    "subscribeThisByEmail": false,
                    "licenseId": 1,
                    "needsReview": true,
                    "parentId": 18514,
                    "fields": {
                        "content": "test content",
                        "title": "test title",
                    },
                },
                "userId": 1,
            }),
        )
        .execute_on(&mut transaction)
        .await;

        let new_entity_id = get_json(mutation_response).await["id"].clone();

        let events_response = Message::new(
            "EventsQuery",
            json!({ "first": 3, "objectId": new_entity_id }),
        )
        .execute_on(&mut transaction)
        .await;
        assert_ok_with(events_response, |result| {
            assert_json_include!(
                actual: &result["events"][0],
                expected: json!({
                    "__typename": "CreateEntityLinkNotificationEvent",
                    "instance": "de",
                    "actorId": 1,
                    "parentId": 18514,
                    "childId": new_entity_id,
                    "objectId": new_entity_id,
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
        })
        .await;
    }
}
