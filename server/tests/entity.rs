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
    use test_utils::*;

    #[actix_rt::test]
    async fn adds_revision() {
        for revision in EntityTestWrapper::all().iter() {
            let mut transaction = begin_transaction().await;

            let mutation_response = Message::new(
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
            .await;

            let revision_id = get_json(mutation_response).await["revisionId"].clone();

            let query_response = Message::new("UuidQuery", json!({ "id": revision_id }))
                .execute_on(&mut transaction)
                .await;

            assert_ok_with(query_response, |result| {
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
            })
            .await;

            assert_event_revision_ok(revision_id, revision.entity_id, &mut transaction).await;
        }
    }

    #[actix_rt::test]
    async fn does_not_add_revision_if_fields_are_same() {
        for revision in EntityTestWrapper::all().iter() {
            let mut transaction = begin_transaction().await;

            let first_mutation_response = Message::new(
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
            .await;

            let first_revision_id = get_json(first_mutation_response).await["revisionId"].clone();
            let first_revision_ids = get_revisions(revision.entity_id, &mut transaction).await;

            let second_mutation_response = Message::new(
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
            .await;

            let second_revision_id = get_json(second_mutation_response).await["revisionId"].clone();
            let second_revision_ids = get_revisions(revision.entity_id, &mut transaction).await;

            assert_eq!(first_revision_id, second_revision_id);
            assert_eq!(first_revision_ids, second_revision_ids);
        }
    }

    async fn get_revisions(id: i32, transaction: &mut sqlx::Transaction<'_, sqlx::MySql>) -> Value {
        let resp = Message::new("UuidQuery", json!({ "id": id }))
            .execute_on(transaction)
            .await;
        get_json(resp).await["revisionIds"].clone()
    }
}

#[cfg(test)]
mod create_mutation {
    use server::uuid::EntityType;
    use test_utils::*;

    #[actix_rt::test]
    async fn creates_entity() {
        for entity in EntityTestWrapper::all().iter() {
            let mut transaction = begin_transaction().await;

            let mutation_response = Message::new(
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
            .await;

            let new_entity_id = get_json(mutation_response).await["id"].clone();

            let query_response = Message::new("UuidQuery", json!({ "id": new_entity_id }))
                .execute_on(&mut transaction)
                .await;

            assert_ok_with(query_response, |result| {
                assert_eq!(
                    from_value_to_entity_type(result["__typename"].clone()),
                    entity.typename
                );
                assert_eq!(result["licenseId"], 1 as i32);
                assert_eq!(result["instance"], "de");
            })
            .await;

            let events_response = Message::new(
                "EventsQuery",
                json!({ "first": 3, "objectId": new_entity_id }),
            )
            .execute_on(&mut transaction)
            .await;

            assert_ok_with(events_response, |result| {
                let (parent_event_name, parent_id, object_id) = match entity.taxonomy_term_id {
                    Some(taxonomy_term_id) => (
                        "CreateTaxonomyLinkNotificationEvent",
                        taxonomy_term_id,
                        taxonomy_term_id,
                    ),
                    None => (
                        "CreateEntityLinkNotificationEvent",
                        entity.parent_id.unwrap(),
                        new_entity_id.as_i64().unwrap() as i32,
                    ),
                };

                assert_json_include!(
                    actual: &result["events"][0],
                    expected: json!({
                        "__typename": "CreateEntityRevisionNotificationEvent",
                        "instance": "de",
                        "actorId": 1,
                        "entityId": new_entity_id
                    })
                );
                assert_json_include!(
                    actual: &result["events"][1],
                    expected: json!({
                        "__typename": "CreateEntityNotificationEvent",
                        "instance": "de",
                        "actorId": 1,
                        "entityId": new_entity_id
                    })
                );
                assert_json_include!(
                    actual: &result["events"][2],
                    expected: json!({
                        "__typename": parent_event_name,
                        "instance": "de",
                        "actorId": 1,
                        "objectId": object_id,
                        "parentId": parent_id,
                        "childId": new_entity_id
                    })
                );
            })
            .await;
        }
    }

    #[actix_rt::test]
    async fn puts_newly_created_entity_as_last_sibling() {
        for entity in EntityTestWrapper::all().iter() {
            let mut transaction = begin_transaction().await;

            let mutation_response = Message::new(
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
            .await;

            let new_entity_id = get_json(mutation_response).await["id"].clone();

            let parent_element_id = match entity.taxonomy_term_id {
                Some(taxonomy_term_id) => taxonomy_term_id,
                None => entity.parent_id.unwrap(),
            };

            let children_ids_name = match entity.typename {
                EntityType::CoursePage => "pageIds",
                EntityType::GroupedExercise => "exerciseIds",
                // The parent of solution, exercise group, doesn't have an array of solutions, just one
                EntityType::Solution => continue,
                _ => "childrenIds",
            };

            let query_response = Message::new("UuidQuery", json!({ "id": parent_element_id }))
                .execute_on(&mut transaction)
                .await;

            assert_ok_with(query_response, |result| {
                let children_ids_value = result[children_ids_name].clone();
                let children_ids = children_ids_value.as_array().unwrap();
                assert_eq!(children_ids[children_ids.len() - 1], new_entity_id);
            })
            .await;
        }
    }

    #[actix_rt::test]
    async fn checkouts_new_revision_when_needs_review_is_true() {
        assert_ok_with(
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
            .await,
            |result| assert!(!result["currentRevisionId"].is_null()),
        )
        .await;
    }

    #[actix_rt::test]
    async fn fails_when_parent_is_no_entity() {
        assert_bad_request(
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
                    "userId": 1 as i32,
                }),
            )
            .execute()
            .await,
            "parent entity with id 1 does not exist",
        )
        .await;
    }

    #[actix_rt::test]
    async fn fails_when_taxonomy_term_does_not_exist() {
        assert_bad_request(
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
                    "userId": 1 as i32,
                }),
            )
            .execute()
            .await,
            "Taxonomy term with id 1 does not exist",
        )
        .await;
    }
}
