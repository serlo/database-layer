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
mod user_delete_regular_users_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn deletes_a_user_permanently() {
        let mut transaction = begin_transaction().await;
        let user_id: i32 = 10;
        let deleted_user_id: i32 = 4;

        let response = Message::new("UserDeleteRegularUsersMutation", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await;

        assert_ok(response, json!({ "success": true, })).await;

        let req = Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await;

        assert_not_found(req).await;

        let deleted_got_activity = Message::new(
            "UserActivityByTypeQuery",
            json!({ "userId": deleted_user_id }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok(
            deleted_got_activity,
            json!({ "edits": 893, "reviews": 923, "comments": 154, "taxonomy": 944 }),
        )
        .await;

        let check_ad = sqlx::query!(r#"select author_id from ad where id = 1"#)
            .fetch_one(&mut transaction)
            .await
            .unwrap();

        assert_eq!(check_ad.author_id as i32, deleted_user_id);

        let check_blog_post = sqlx::query!(r#"select author_id from blog_post where id = 1199"#)
            .fetch_one(&mut transaction)
            .await
            .unwrap();

        assert_eq!(check_blog_post.author_id as i32, deleted_user_id);

        let check_comment = sqlx::query!(r#"select author_id from comment where id = 16740"#)
            .fetch_one(&mut transaction)
            .await
            .unwrap();

        assert_eq!(check_comment.author_id as i32, deleted_user_id);

        let comment_vote_does_not_exist =
            sqlx::query!(r#"select * from comment_vote where user_id = 10"#)
                .fetch_optional(&mut transaction)
                .await
                .unwrap();

        assert!(comment_vote_does_not_exist.is_none());

        let check_entity_revision =
            sqlx::query!(r#"select author_id from entity_revision where id = 16114"#)
                .fetch_one(&mut transaction)
                .await
                .unwrap();

        assert_eq!(check_entity_revision.author_id as i32, deleted_user_id);

        let check_event_log = sqlx::query!(r#"select actor_id from event_log where id = 38383"#)
            .fetch_one(&mut transaction)
            .await
            .unwrap();

        assert_eq!(check_event_log.actor_id as i32, deleted_user_id);

        let notification_does_not_exist =
            sqlx::query!(r#"select * from notification where user_id = 10"#)
                .fetch_optional(&mut transaction)
                .await
                .unwrap();

        assert!(notification_does_not_exist.is_none());

        let check_page_revision =
            sqlx::query!(r#"select author_id from page_revision where id = 16283"#)
                .fetch_one(&mut transaction)
                .await
                .unwrap();

        assert_eq!(check_page_revision.author_id as i32, deleted_user_id);

        let role_user_does_not_exist =
            sqlx::query!(r#"select * from role_user where user_id = 10"#)
                .fetch_optional(&mut transaction)
                .await
                .unwrap();

        assert!(role_user_does_not_exist.is_none());

        let subscription_does_not_exist =
            sqlx::query!(r#"select * from subscription where user_id = 10"#)
                .fetch_optional(&mut transaction)
                .await
                .unwrap();

        assert!(subscription_does_not_exist.is_none());

        let subscription_does_not_exist2 =
            sqlx::query!(r#"select * from subscription where uuid_id = 10"#)
                .fetch_optional(&mut transaction)
                .await
                .unwrap();

        assert!(subscription_does_not_exist2.is_none());

        let uuid_does_not_exist = sqlx::query!(r#"select * from uuid where id = 10"#)
            .fetch_optional(&mut transaction)
            .await
            .unwrap();

        assert!(uuid_does_not_exist.is_none());
    }

    #[actix_rt::test]
    async fn flag_is_deleted() {
        let mut transaction = begin_transaction().await;
        let reporter_id: i32 = 15474;

        Message::new(
            "UserDeleteRegularUsersMutation",
            json!({ "id": reporter_id }),
        )
        .execute_on(&mut transaction)
        .await;

        let flag_does_not_exist = sqlx::query!(r#"select * from flag where reporter_id = 10"#)
            .fetch_optional(&mut transaction)
            .await
            .unwrap();

        assert!(flag_does_not_exist.is_none());
    }

    #[actix_rt::test]
    async fn fails_when_user_does_not_exist() {
        let mut transaction = begin_transaction().await;
        let user_id: i32 = -1;

        let response = Message::new("UserDeleteRegularUsersMutation", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await;

        assert_bad_request(response, "The requested user does not exist.").await;
    }

    #[actix_rt::test]
    async fn fails_when_trying_to_delete_deleted() {
        let mut transaction = begin_transaction().await;
        let deleted_user_id: i32 = 4;

        let response = Message::new(
            "UserDeleteRegularUsersMutation",
            json!({ "id": deleted_user_id }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_bad_request(response, "You cannot delete the user Deleted.").await;
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

        let mutation_response = Message::new(
            "UserSetDescriptionMutation",
            json!({ "userId": user_id, "description": "new description".to_string() }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok(mutation_response, json!({ "success": true })).await;

        let query_response = Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await;

        assert_ok_with(query_response, |result| {
            assert_eq!(result["description"], "new description".to_string())
        })
        .await;
    }
}

mod user_set_email_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn updates_user_email() {
        let mut transaction = begin_transaction().await;
        let user_id = create_new_test_user(&mut transaction).await.unwrap();
        let new_email = "user@example.com".to_string();
        let user = get_json(
            Message::new("UuidQuery", json!({ "id": user_id }))
                .execute_on(&mut transaction)
                .await,
        )
        .await;
        let username = user.get("username").unwrap().as_str().unwrap();

        let mutation_response = Message::new(
            "UserSetEmailMutation",
            json!({ "userId": user_id, "email": &new_email }),
        )
        .execute_on(&mut transaction)
        .await;

        assert_ok(
            mutation_response,
            json!({ "success": true, "username": username }),
        )
        .await;

        let email = get_email(user_id, &mut transaction).await.unwrap();

        assert_eq!(email, new_email)
    }
}
