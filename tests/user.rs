#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use serde_json::json;
    use std::str::from_utf8;

    use rand::{distributions::Alphanumeric, Rng};
    use serlo_org_database_layer::database::Executor;
    use serlo_org_database_layer::{configure_app, create_database_pool};

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
                "comments": 37,
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
                "payload": { "userIds": [user_id] }
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

    // TODO: Move this utility function into an extra crate
    async fn create_new_test_user<'a, E>(executor: E) -> Result<i32, sqlx::Error>
    where
        E: Executor<'a>,
    {
        let mut transaction = executor.begin().await?;

        sqlx::query!(
            r#"
                INSERT INTO uuid (trashed, discriminator) VALUES (0, "user")
            "#
        )
        .execute(&mut transaction)
        .await?;

        let new_user_id = sqlx::query!("SELECT LAST_INSERT_ID() as id FROM uuid")
            .fetch_one(&mut transaction)
            .await?
            .id as i32;

        sqlx::query!(
            r#"
                INSERT INTO user (id, username, email, password, token)
                VALUES (?, ?, ?, "", ?)
            "#,
            new_user_id,
            random_string(10),
            random_string(10),
            random_string(10)
        )
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;

        Ok(new_user_id)
    }

    fn random_string(nr: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(nr)
            .map(char::from)
            .collect()
    }
}
