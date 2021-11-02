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
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(response, |result| assert_eq!(result["cohesive"], true)).await;
    }

    #[actix_rt::test]
    async fn taxonomy_terms_return_term_id() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let message = UuidMessage::UuidQuery(uuid_query::Payload { id: 1385 });
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&message)
            .to_request();
        let resp = test::call_service(&app, req).await;

        let uuid = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(uuid["taxonomyId"], 4);
    }
}
