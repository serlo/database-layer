#[cfg(test)]
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

#[cfg(test)]
mod add_revision_mutation {
    use test_utils::*;

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

    async fn get_revisions(id: i32, transaction: &mut sqlx::Transaction<'_, sqlx::MySql>) -> Value {
        Message::new("UuidQuery", json!({ "id": id }))
            .execute_on(transaction)
            .await
            .get_json()["revisionIds"]
            .clone()
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

            Message::new("UuidQuery", json!({ "id": new_entity_id }))
                .execute_on(&mut transaction)
                .await
                .should_be_ok_with(|result| {
                    assert_eq!(
                        from_value_to_entity_type(result["__typename"].clone()),
                        entity.typename
                    );
                    assert_eq!(result["licenseId"], 1 as i32);
                    assert_eq!(result["instance"], "de");
                });

            Message::new(
                "EventsQuery",
                json!({ "first": 3, "objectId": new_entity_id }),
            )
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
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
            });
        }
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
                "userId": 1 as i32,
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
                "userId": 1 as i32,
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

#[cfg(test)]
mod deleted_entities_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn gives_back_first_deleted_entities() {
        let first: i32 = 3;

        Message::new("DeletedEntitiesQuery", json!({ "first": first }))
            .execute()
            .await
            .should_be_ok_with(|result| {
                assert_has_length(&result["deletedEntities"], first as usize);
                assert_eq!(
                    result["deletedEntities"][0],
                    json!({ "id": 17635, "dateOfDeletion": "2014-03-13T15:16:01+01:00" })
                );
            });
    }

    #[actix_rt::test]
    async fn gives_back_first_deleted_entities_after_date() {
        let date = "2014-08-01T00:00:00+02:00";

        Message::new("DeletedEntitiesQuery", json!({ "first": 4, "after": date }))
            .execute()
            .await
            .should_be_ok_with(|result| {
                assert_eq!(
                    result["deletedEntities"][0],
                    json!({ "id": 28067, "dateOfDeletion": "2014-08-28T08:38:51+02:00" })
                );
            });
    }

    #[actix_rt::test]
    async fn gives_back_first_deleted_entities_after_date_with_small_time_differences() {
        let date = "2014-03-24T14:23:20+01:00";

        Message::new(
            "DeletedEntitiesQuery",
            json!({ "first": 1 as i32, "after": date }),
        )
        .execute()
        .await
        .should_be_ok_with(|result| {
            assert_eq!(
                result["deletedEntities"][0],
                json!({ "id": 19595 as i32, "dateOfDeletion": "2014-03-24T14:23:28+01:00" })
            );
        });
    }

    #[actix_rt::test]
    async fn gives_back_first_deleted_entities_of_instance() {
        Message::new(
            "DeletedEntitiesQuery",
            json!({ "first": 4, "instance": "de" }),
        )
        .execute()
        .await
        .should_be_ok_with(|result| {
            assert_eq!(
                result["deletedEntities"][0],
                json!({ "id": 17740, "dateOfDeletion": "2014-03-13T17:13:05+01:00" })
            );
        });
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

#[cfg(test)]
mod set_license_mutation {
    use test_utils::*;

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
            json!({ "first": 1 as usize, "objectId": entity_id as usize}),
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
            json!({ "first": 1 as usize, "objectId": entity_id as usize}),
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
