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
                        "instance": "de",
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
                        "__typename": parent_event_name,
                        "instance": "de",
                        "actorId": 1,
                        "objectId": object_id,
                        "parentId": parent_id,
                        "childId": new_entity_id
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
                        "instance": "de",
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
}
