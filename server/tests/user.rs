#[cfg(test)]
mod tests {
    use actix_web::body::to_bytes;
    use actix_web::{test, App};
    use serde_json::{from_slice, from_str, json, Value};
    use std::str::from_utf8;

    use server::{configure_app, create_database_pool};
    use test_utils::{create_new_test_user, handle_message, set_description};

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
        let mut transaction = create_database_pool().await.unwrap().begin().await.unwrap();

        let user_id = create_new_test_user(&mut transaction).await.unwrap();
        set_description(user_id, "Test", &mut transaction)
            .await
            .unwrap();
        let user_id2 = create_new_test_user(&mut transaction).await.unwrap();
        set_description(user_id2, "Test", &mut transaction)
            .await
            .unwrap();

        let resp = handle_message(
            &mut transaction,
            "UserPotentialSpamUsersQuery",
            json!({ "first": 10 }),
        )
        .await;

        assert!(resp.status().is_success());

        let result: Value = from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(result, json!({ "userIds": [user_id2, user_id] }));

        let resp = handle_message(
            &mut transaction,
            "UserPotentialSpamUsersQuery",
            json!({ "first": 10, "after": user_id2 }),
        )
        .await;

        assert!(resp.status().is_success());

        let result: Value = from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(result, json!({ "userIds": [user_id] }));
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
