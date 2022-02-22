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
    use serde_json::Value::Null;
    use server::uuid::abstract_entity_revision::EntityRevisionType;
    use test_utils::*;

    #[actix_rt::test]
    async fn adds_applet_revision() {
        let mut transaction = begin_transaction().await;

        let response = Message::new(
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

        assert_ok(response, json!({ "success": true, "reason": Null })).await;

        let revision_id = sqlx::query!(r#"SELECT id FROM entity_revision ORDER BY id desc limit 1"#)
            .fetch_one(&mut transaction)
            .await
            .unwrap()
            .id as i32;

        let query_response = Message::new("UuidQuery", json!({ "id": revision_id }))
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
}
