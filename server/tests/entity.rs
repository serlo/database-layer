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
