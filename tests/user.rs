#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use futures::StreamExt;
    use serde_json::json;

    use serlo_org_database_layer::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn user_activity_by_type() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let mut app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({
                "type": "UserActivityByTypeQuery",
                "payload": { "userId": 1 }
            }))
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());

        let (bytes, _) = resp.take_body().into_future().await;
        let result = json::parse(std::str::from_utf8(&bytes.unwrap().unwrap()).unwrap()).unwrap();
        assert_eq!(
            result,
            json::object! {
                "edits": 209,
                "reviews": 213,
                "comments": 37,
                "taxonomy": 836
            }
        );
    }
}
