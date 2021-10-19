#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use serde_json::json;
    use std::str::from_utf8;

    use server::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn start_thread_rejects_when_content_is_empty() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .append_header(("Rollback", "true"))
            .set_json(&json!({
                "type": "ThreadCreateThreadMutation",
                "payload": {
                    "title": "title",
                    "content": "",
                    "objectId": 1565,
                    "userId": 1,
                    "subscribe": true,
                    "sendEmail": false,
                }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(
            result,
            json::object! {
                "success": false,
                "reason": "Cannot create thread: content is empty",
            }
        );
    }

    #[actix_rt::test]
    async fn create_comment_rejects_when_content_is_empty() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .append_header(("Rollback", "true"))
            .set_json(&json!({
                "type": "ThreadCreateCommentMutation",
                "payload": {
                    "threadId": 17774,
                    "userId": 1,
                    "content": "",
                    "subscribe": true,
                    "sendEmail": false,
                }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(
            result,
            json::object! {
                "success": false,
                "reason": "Cannot create comment: content is empty",
            }
        );
    }
}
