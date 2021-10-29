#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::str::from_utf8;
    use test_utils::*;

    use server::uuid::{uuid_query, UuidMessage};
    use server::{configure_app, create_database_pool};
    use test_utils::set_entity_revision_field;

    #[actix_rt::test]
    async fn test_pact1() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let message = UuidMessage::UuidQuery(uuid_query::Payload { id: 1 });
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&message)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let uuid = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(uuid["__typename"], "User");
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
