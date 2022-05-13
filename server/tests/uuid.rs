#[cfg(test)]
mod uuid_query {
    use actix_web::{test, App};
    use test_utils::*;

    use server::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn test_pact1() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({ "type": "UuidQuery", "payload": { "id": 1 } }))
            .to_request();
        let response = test::call_service(&app, req).await;

        assert_ok_with(response.into(), |result| {
            assert_eq!(result["__typename"], "User")
        })
        .await;
    }

    #[actix_rt::test]
    async fn returns_property_cohesive_on_exercise_groups() {
        let mut transaction = begin_transaction().await;

        let exercise_group_revision_id = 26070;
        set_entity_revision_field(
            exercise_group_revision_id,
            "cohesive",
            "true",
            &mut transaction,
        )
        .await
        .unwrap();

        let response = Message::new("UuidQuery", json!({ "id": exercise_group_revision_id }))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(response, |result| assert_eq!(result["cohesive"], true)).await;
    }

    #[actix_rt::test]
    async fn returns_property_taxonomy_id_on_taxonomy_terms() {
        let response = Message::new("UuidQuery", json!({ "id": 1385 }))
            .execute()
            .await;

        assert_ok_with(response, |result| assert_eq!(result["taxonomyId"], 4)).await;
    }
}

#[cfg(test)]
mod set_uuid_state_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn fails_for_untrashable_uuids() {
        for discriminator in ["entityRevision", "user"].iter() {
            let mut transaction = begin_transaction().await;

            let revision_id = sqlx::query!(
                "select id from uuid where discriminator = ? and trashed = false",
                discriminator
            )
            .fetch_one(&mut transaction)
            .await
            .unwrap()
            .id as i32;

            let response = Message::new(
                "UuidSetStateMutation",
                json!({ "ids": [revision_id], "userId": 1, "trashed": true }),
            )
            .execute()
            .await;

            assert_bad_request(response).await;
        }
    }
}
