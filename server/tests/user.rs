#[cfg(test)]
mod tests {
    use serde_json::json;
    use test_utils::*;

    #[actix_rt::test]
    async fn user_activity_by_type() {
        let response = Message::new("UserActivityByTypeQuery", json!({ "userId": 1 }))
            .execute()
            .await;

        assert_ok(
            response,
            json!({ "edits": 209, "reviews": 213, "comments": 62, "taxonomy": 836 }),
        )
        .await;
    }

    #[actix_rt::test]
    async fn user_delete_bots_mutation() {
        let mut transaction = begin_transaction().await;
        let user_id = create_new_test_user(&mut transaction).await.unwrap();

        let response = Message::new("UserDeleteBotsMutation", json!({ "botIds": [user_id] }))
            .execute_on(&mut transaction)
            .await;
        assert_ok(response, json!({ "success": true })).await;

        let req = Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await;

        assert_not_found(req).await;
    }

    #[actix_rt::test]
    async fn user_potential_spam_users_query() {
        let mut transaction = begin_transaction().await;

        let user_id = create_new_test_user(&mut transaction).await.unwrap();
        set_description(user_id, "Test", &mut transaction)
            .await
            .unwrap();

        let response = Message::new("UserPotentialSpamUsersQuery", json!({ "first": 10 }))
            .execute_on(&mut transaction)
            .await;

        assert_ok(response, json!({ "userIds": [user_id] })).await;
    }

    #[actix_rt::test]
    async fn user_potential_spam_users_query_with_after_parameter() {
        let mut transaction = begin_transaction().await;

        let user_id = create_new_test_user(&mut transaction).await.unwrap();
        set_description(user_id, "Test", &mut transaction)
            .await
            .unwrap();
        let user_id2 = create_new_test_user(&mut transaction).await.unwrap();
        set_description(user_id2, "Test", &mut transaction)
            .await
            .unwrap();

        let response = Message::new(
            "UserPotentialSpamUsersQuery",
            json!({ "first": 10, "after": user_id2 }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok(response, json!({ "userIds": [user_id] })).await;
    }

    #[actix_rt::test]
    async fn potential_spam_users_query_fails_when_first_parameter_is_too_high() {
        let response = Message::new("UserPotentialSpamUsersQuery", json!({ "first": 1_000_000 }))
            .execute()
            .await;

        assert_bad_request(response, "parameter `first` is too high").await;
    }
}
