#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use futures::StreamExt;

    use serlo_org_database_layer::create_database_pool;

    #[actix_rt::test]
    async fn test_pact1() {
        let pool = create_database_pool().await.unwrap();
        let app = App::new()
            .data(pool.clone())
            .configure(serlo_org_database_layer::uuid::init);
        let mut app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/uuid/1").to_request();
        let mut resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());

        // first chunk
        let (bytes, _) = resp.take_body().into_future().await;
        let uuid = json::parse(std::str::from_utf8(&bytes.unwrap().unwrap()).unwrap()).unwrap();
        assert_eq!(uuid["__typename"], "User");
    }
}
