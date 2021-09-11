#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::str::from_utf8;

    use serlo_org_database_layer::uuid::{uuid_query, UuidMessage};
    use serlo_org_database_layer::{configure_app, create_database_pool};

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
}
