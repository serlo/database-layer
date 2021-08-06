#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::str::from_utf8;

    use serlo_org_database_layer::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn subjects_query() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "SubjectsQuery",
                "payload": {}
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            result,
            json::object! {
              "subjects": [
                {
                  "instance": "de",
                  "taxonomyTermId": 5
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 17744
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 18230
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 23362
                },
                {
                  "instance": "en",
                  "taxonomyTermId": 23593
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 25712
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 25979
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 26523
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 33894
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 35608
                }
              ]
            }
        );
    }
}
