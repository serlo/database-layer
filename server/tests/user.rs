#[cfg(test)]
mod user_activity_by_type_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_user_activity() {
        let response = Message::new("UserActivityByTypeQuery", json!({ "userId": 1 }))
            .execute()
            .await;

        assert_ok(
            response,
            json!({ "edits": 209, "reviews": 213, "comments": 62, "taxonomy": 836 }),
        )
        .await;
    }
}

#[cfg(test)]
mod user_delete_bots_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn deletes_a_user_permanentely() {
        let mut transaction = begin_transaction().await;
        let user_id = create_new_test_user(&mut transaction).await.unwrap();

        set_email(user_id, "testuser@example.org", &mut transaction)
            .await
            .unwrap();

        let response = Message::new("UserDeleteBotsMutation", json!({ "botIds": [user_id] }))
            .execute_on(&mut transaction)
            .await;
        assert_ok(
            response,
            json!({ "success": true, "emailHashes": ["cd5610c5b6be1e5a62fb621031ae3856"] }),
        )
        .await;

        let req = Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await;

        assert_not_found(req).await;
    }
}

#[cfg(test)]
mod user_potential_spam_users_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_user_with_a_description() {
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
    async fn with_after_parameter() {
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
    async fn fails_when_first_parameter_is_too_high() {
        let response = Message::new("UserPotentialSpamUsersQuery", json!({ "first": 1_000_000 }))
            .execute()
            .await;

        assert_bad_request(response, "parameter `first` is too high").await;
    }
}

mod user_set_description_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn updates_user_description() {
        let mut transaction = begin_transaction().await;
        let user_id = create_new_test_user(&mut transaction).await.unwrap();

        let response = Message::new(
            "UserSetDescriptionMutation",
            json!({ "userId": [user_id], "description": "new description".to_string() }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok(response, json!({ "success": true })).await;

        let query_response = Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["description"], "new description".to_string())
        })
        .await;
    }
}
