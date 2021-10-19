#![allow(clippy::bool_assert_comparison)]
#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::str::from_utf8;

    use server::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn set_uuid_state_for_untrashable_uuids_fails() {
        for discriminator in ["entityRevision", "user"].iter() {
            let pool = create_database_pool().await.unwrap();
            let mut transaction = pool.begin().await.unwrap();

            let app = configure_app(App::new(), pool);
            let app = test::init_service(app).await;

            let revision_id = sqlx::query!(
                "select id from uuid where discriminator = ? and trashed = false",
                discriminator
            )
            .fetch_one(&mut transaction)
            .await
            .unwrap()
            .id as i32;

            let req = test::TestRequest::post()
                .uri("/")
                .set_json(&serde_json::json!({
                    "type": "UuidSetStateMutation",
                    "payload": {
                        "ids": vec![revision_id],
                        "userId": 1,
                        "trashed": true,
                    }
                }))
                .to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), 400);

            let result = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();
            assert_eq!(result["success"], false);
            assert_eq!(
                result["reason"],
                format!(
                    "uuid {} with type \"{}\" cannot be deleted via a setState mutation",
                    revision_id, discriminator
                )
            );
        }
    }
}
