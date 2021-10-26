#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::str::from_utf8;

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
        let pool = create_database_pool().await.unwrap();

        let exercise_group_revision_id = 26070;

        set_entity_revision_field(exercise_group_revision_id, "cohesive", "true", &pool)
            .await
            .unwrap();

        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let message = UuidMessage::UuidQuery(uuid_query::Payload {
            id: exercise_group_revision_id,
        });
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&message)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let uuid = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(uuid["cohesive"], true);
    }
}
