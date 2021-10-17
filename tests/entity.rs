// TODO?: test the filters?

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::str::from_utf8;

    use serlo_org_database_layer::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn unrevised_entities_query() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "UnrevisedEntitiesQuery",
                "payload": {}
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            result,
            json::object! {
                "unrevisedEntityIds": [
                    26892,
                    33582,
                    34741,
                    34907,
                    35247,
                    35556
                 ]
            }
        );
    }

    #[actix_rt::test]
    #[allow(clippy::bool_assert_comparison)]
    async fn entities_query_without_after_parameter() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EntitiesQuery",
                "payload": { "first": 10, "lastModified": "2014-01-01" }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let entities = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(entities["entityIds"].len(), 10);

        assert_eq!(
            entities["entityIds"],
            json::array![ 2219, 2221, 2225, 2227, 2229, 2231, 2233, 2235, 2239, 2241 ]
        );
    }

    #[actix_rt::test]
    #[allow(clippy::bool_assert_comparison)]
    async fn entities_query_with_after_parameter() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EntitiesQuery",
                "payload": { "first": 10, "lastModified": "2014-01-01", "after": 2241 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let entities =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            entities["entityIds"],
            json::array![ 2243, 2245, 2247, 2249, 2251, 2253, 2255, 2257, 2259, 2261 ]
        );
    }

    #[actix_rt::test]
    #[allow(clippy::bool_assert_comparison)]
    async fn entities_query_with_instance_paramenter() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EntitiesQuery",
                "payload": { "first": 10, "lastModified": "2014-01-01", "instance": "en" }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let entities =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            entities["entityIds"],
            json::array![ 34124, 35574, 35575, 35581, 35582, 35583, 35584, 35585, 35586, 35587 ]
        );
    }

    #[actix_rt::test]
    async fn entities_query_fails_when_first_parameter_is_too_high() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EntitiesQuery",
                "payload": { "first": 10_000, "lastModified": "2014-01-01" }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);

        let entities =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            entities,
            json::object! { "success": false, "reason": "The 'first' value should be less than 10_000" }
        );
    }

    #[actix_rt::test]
    async fn entities_query_fails_when_last_modified_parameter_is_missing() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EntitiesQuery",
                "payload": { "first": 10 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);

        let entities =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            entities,
            json::object! { "success": false, "reason": "The key 'lastModified' is required. Note: 'last_modified' is not accepted" }
        );
    }

    #[actix_rt::test]
    async fn entities_query_fails_when_first_parameter_is_missing() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EntitiesQuery",
                "payload": { "lastModified": "2014-01-01" }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);

        let entities =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            entities,
            json::object! { "success": false, "reason": "The 'first' key is required" }
        );
    }
}
