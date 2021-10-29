#[cfg(test)]
mod tests {
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
    async fn exercise_group_property_cohesive() {
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
            .execute()
            .await;

        assert_ok_with(response, |result| assert_eq!(result["cohesive"], true)).await;
    }
}
