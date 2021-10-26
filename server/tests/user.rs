#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use serde_json::json;
    use std::str::from_utf8;

    use server::{configure_app, create_database_pool};
    use test_utils::{create_new_test_user, delete_all_test_user, set_description};

    #[actix_rt::test]
    async fn user_activity_by_type() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({
                "type": "UserActivityByTypeQuery",
                "payload": { "userId": 1 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(
            result,
            json::object! {
                "edits": 209,
                "reviews": 213,
                "comments": 62,
                "taxonomy": 836
            }
        );
    }

    #[actix_rt::test]
    async fn user_delete_bots_mutation() {
        let pool = create_database_pool().await.unwrap();
        let user_id = create_new_test_user(&pool).await.unwrap();

        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({
                "type": "UserDeleteBotsMutation",
                "payload": { "botIds": [user_id] }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(result, json::object! { "success": true });

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({
                "type": "UuidQuery",
                "payload": { "id": user_id }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 404);
    }

    #[actix_rt::test]
    async fn user_potential_spam_users_query() {
        let pool = create_database_pool().await.unwrap();

        delete_all_test_user(&pool).await.unwrap();
        let user_id = create_new_test_user(&pool).await.unwrap();
        set_description(user_id, "Test", &pool).await.unwrap();
        let user_id2 = create_new_test_user(&pool).await.unwrap();
        set_description(user_id2, "Test", &pool).await.unwrap();

        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({
                "type": "UserPotentialSpamUsersQuery",
                "payload": { "first": 10 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        assert_eq!(result, json::object! { "userIds": [user_id2, user_id] });

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({
                "type": "UserPotentialSpamUsersQuery",
                "payload": { "first": 10, "after": user_id2 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
        dbg!(&result);
        assert_eq!(result, json::object! { "userIds": [user_id] });
    }

    #[actix_rt::test]
    async fn potential_spam_users_query_fails_when_first_parameter_is_too_high() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "UserPotentialSpamUsersQuery",
                "payload": { "first": 1_000_000 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);

        let result =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            result,
            json::object! { "success": false, "reason": "parameter `first` is too high" }
        );
    }
}
