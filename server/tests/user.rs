mod user_activity_by_type_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_user_activity() {
        Message::new("UserActivityByTypeQuery", json!({ "userId": 1 }))
            .execute()
            .await
            .should_be_ok_with_body(
                json!({ "edits": 197, "reviews": 201, "comments": 62, "taxonomy": 836 }),
            );
    }
}

mod user_add_role_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn adds_role_to_user() {
        let username = "1229fb21";
        let user_id: i32 = 98;
        let role_name = "sysadmin";
        let role_id: i32 = 11;
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserAddRoleMutation",
            json!({ "username": username, "roleName": role_name}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();

        let response = sqlx::query!(
            r#"
                SELECT user_id
                FROM role_user
                WHERE user_id = ?
                and role_id = ?
            "#,
            user_id,
            role_id
        )
        .fetch_all(&mut *transaction)
        .await
        .unwrap();
        assert_eq!(response[0].user_id as i32, user_id);
    }

    #[actix_rt::test]
    async fn does_not_add_a_new_row_if_user_already_has_role() {
        let user_id: i32 = 1;
        let username = "admin";
        let role_name = "sysadmin";
        let role_id: i32 = 11;
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserAddRoleMutation",
            json!({ "username": username, "roleName": role_name}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();

        let response = sqlx::query!(
            r#"
                SELECT user_id
                FROM role_user
                WHERE user_id = ?
                and role_id = ?
            "#,
            user_id,
            role_id
        )
        .fetch_all(&mut *transaction)
        .await
        .unwrap();

        assert_eq!(response.len(), 1);
    }

    #[actix_rt::test]
    async fn should_throw_bad_request_for_non_existent_role() {
        let username = "admin";
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserAddRoleMutation",
            json!({ "username": username, "roleName": "not a role"}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn should_throw_bad_request_for_non_existent_user() {
        let username = "not a user";
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserAddRoleMutation",
            json!({ "username": username, "roleName": "login"}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_bad_request();
    }
}

mod user_create_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn creates_an_user() {
        let mut transaction = begin_transaction().await;
        let response = Message::new("UserCreateMutation", json!({ "username": "testUser", "email": "mail@mail.test", "password": "securePassword!"}))
            .execute_on(&mut transaction)
            .await
            .get_json();

        let last_id = sqlx::query!(
            r#"
                SELECT LAST_INSERT_ID() AS id
                FROM uuid;
            "#,
        )
        .fetch_one(&mut *transaction)
        .await
        .unwrap()
        .id;

        let user_id = &response["userId"];

        assert_eq!(user_id, last_id);

        let user = Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await
            .get_json();

        assert_eq!(user["username"], "testUser");
        assert_eq!(user["roles"][0], "login");
    }

    #[actix_rt::test]
    async fn fails_when_username_too_long() {
        Message::new("UserCreateMutation", json!({ "username": "testUsertestUsertestUsertestUser!", "email": "mail@mail.test", "password": "securePassword!"}))
            .execute()
            .await
            .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_email_too_long() {
        Message::new("UserCreateMutation", json!({ "username": "testUser", "email": "mail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail.testmail@mail.test", "password": "securePassword!"}))
            .execute()
            .await
            .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_password_too_long() {
        Message::new("UserCreateMutation", json!({ "username": "testUser!", "email": "mail@mail.test", "password": "securePassword!securePassword!securePassword!123456"}))
            .execute()
            .await
            .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_username_is_empty_string() {
        Message::new(
            "UserCreateMutation",
            json!({ "username": "  ", "email": "mail@mail.test", "password": "securePassword"}),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

mod user_delete_bots_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn deletes_a_user_permanentely() {
        let mut transaction = begin_transaction().await;
        let user_id = create_new_test_user(&mut *transaction).await.unwrap();

        set_email(user_id, "testuser@example.org", &mut *transaction)
            .await
            .unwrap();

        Message::new("UserDeleteBotsMutation", json!({ "botIds": [user_id] }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with_body(
                json!({ "success": true, "emailHashes": ["cd5610c5b6be1e5a62fb621031ae3856"] }),
            );

        Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_not_found();
    }
}

mod user_delete_regular_users_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn deletes_a_user_permanently() {
        let mut transaction = begin_transaction().await;
        let user_id: i32 = 10;
        let deleted_user_id: i32 = 4;

        Message::new(
            "UserDeleteRegularUsersMutation",
            json!({ "userId": user_id }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "success": true, }));

        Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_not_found();

        Message::new(
            "UserActivityByTypeQuery",
            json!({ "userId": deleted_user_id }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(
            json!({ "edits": 893, "reviews": 923, "comments": 154, "taxonomy": 944 }),
        );

        let check_ad = sqlx::query!(r#"select author_id from ad where id = 1"#)
            .fetch_one(&mut *transaction)
            .await
            .unwrap();

        assert_eq!(check_ad.author_id as i32, deleted_user_id);

        let check_blog_post = sqlx::query!(r#"select author_id from blog_post where id = 1199"#)
            .fetch_one(&mut *transaction)
            .await
            .unwrap();

        assert_eq!(check_blog_post.author_id as i32, deleted_user_id);

        let check_comment = sqlx::query!(r#"select author_id from comment where id = 16740"#)
            .fetch_one(&mut *transaction)
            .await
            .unwrap();

        assert_eq!(check_comment.author_id as i32, deleted_user_id);

        let comment_vote_does_not_exist =
            sqlx::query!(r#"select * from comment_vote where user_id = 10"#)
                .fetch_optional(&mut *transaction)
                .await
                .unwrap();

        assert!(comment_vote_does_not_exist.is_none());

        let check_entity_revision =
            sqlx::query!(r#"select author_id from entity_revision where id = 16114"#)
                .fetch_one(&mut *transaction)
                .await
                .unwrap();

        assert_eq!(check_entity_revision.author_id as i32, deleted_user_id);

        let check_event_log = sqlx::query!(r#"select actor_id from event_log where id = 38383"#)
            .fetch_one(&mut *transaction)
            .await
            .unwrap();

        assert_eq!(check_event_log.actor_id as i32, deleted_user_id);

        let notification_does_not_exist =
            sqlx::query!(r#"select * from notification where user_id = 10"#)
                .fetch_optional(&mut *transaction)
                .await
                .unwrap();

        assert!(notification_does_not_exist.is_none());

        let check_page_revision =
            sqlx::query!(r#"select author_id from page_revision where id = 16283"#)
                .fetch_one(&mut *transaction)
                .await
                .unwrap();

        assert_eq!(check_page_revision.author_id as i32, deleted_user_id);

        let role_user_does_not_exist =
            sqlx::query!(r#"select * from role_user where user_id = 10"#)
                .fetch_optional(&mut *transaction)
                .await
                .unwrap();

        assert!(role_user_does_not_exist.is_none());

        let subscription_does_not_exist =
            sqlx::query!(r#"select * from subscription where user_id = 10"#)
                .fetch_optional(&mut *transaction)
                .await
                .unwrap();

        assert!(subscription_does_not_exist.is_none());

        let subscription_does_not_exist2 =
            sqlx::query!(r#"select * from subscription where uuid_id = 10"#)
                .fetch_optional(&mut *transaction)
                .await
                .unwrap();

        assert!(subscription_does_not_exist2.is_none());

        let uuid_does_not_exist = sqlx::query!(r#"select * from uuid where id = 10"#)
            .fetch_optional(&mut *transaction)
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
            json!({ "userId": reporter_id }),
        )
        .execute_on(&mut transaction)
        .await;

        let flag_does_not_exist = sqlx::query!(r#"select * from flag where reporter_id = 10"#)
            .fetch_optional(&mut *transaction)
            .await
            .unwrap();

        assert!(flag_does_not_exist.is_none());
    }

    #[actix_rt::test]
    async fn fails_when_user_does_not_exist() {
        Message::new("UserDeleteRegularUsersMutation", json!({ "userId": -1 }))
            .execute()
            .await
            .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn fails_when_trying_to_delete_deleted() {
        Message::new("UserDeleteRegularUsersMutation", json!({ "userId": 4 }))
            .execute()
            .await
            .should_be_bad_request();
    }
}

mod user_potential_spam_users_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_user_with_a_description() {
        let mut transaction = begin_transaction().await;

        let user_id = create_new_test_user(&mut *transaction).await.unwrap();
        set_description(user_id, "Test", &mut *transaction)
            .await
            .unwrap();

        Message::new("UserPotentialSpamUsersQuery", json!({ "first": 2 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with_body(json!({ "userIds": [user_id] }));
    }

    #[actix_rt::test]
    async fn with_after_parameter() {
        let mut transaction = begin_transaction().await;

        let user_id = create_new_test_user(&mut *transaction).await.unwrap();
        set_description(user_id, "Test", &mut *transaction)
            .await
            .unwrap();
        let user_id2 = create_new_test_user(&mut *transaction).await.unwrap();
        set_description(user_id2, "Test", &mut *transaction)
            .await
            .unwrap();

        Message::new(
            "UserPotentialSpamUsersQuery",
            json!({ "first": 3, "after": user_id2 }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "userIds": [user_id] }));
    }

    #[actix_rt::test]
    async fn does_not_return_user_with_higher_role() {
        let mut transaction = begin_transaction().await;

        let user_id = create_new_test_user(&mut *transaction).await.unwrap();
        let user_id2 = create_new_test_user(&mut *transaction).await.unwrap();
        set_description(user_id, "Test", &mut *transaction)
            .await
            .unwrap();
        set_description(user_id2, "Test", &mut *transaction)
            .await
            .unwrap();
        sqlx::query!(
            r#"
                INSERT INTO role_user (user_id, role_id)
                VALUES (?, 3)
            "#,
            user_id2
        )
        .execute(&mut *transaction)
        .await
        .unwrap();

        Message::new("UserPotentialSpamUsersQuery", json!({ "first": 1 }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with_body(json!({ "userIds": [user_id] }));
    }

    #[actix_rt::test]
    async fn does_not_return_user_with_6_edits() {
        let mut transaction = begin_transaction().await;

        let user_id = create_new_test_user(&mut *transaction).await.unwrap();
        let user_id2 = create_new_test_user(&mut *transaction).await.unwrap();
        set_description(user_id, "Test", &mut *transaction)
            .await
            .unwrap();
        set_description(user_id2, "Test", &mut *transaction)
            .await
            .unwrap();

        for a in 0..6_i32 {
            Message::new(
                "PageAddRevisionMutation",
                json!({
                    "title": format!("title{a}"),
                    "content": format!("content{a}"),
                    "userId": user_id2,
                    "pageId": 16569
                }),
            )
            .execute_on(&mut transaction)
            .await
            .should_be_ok();
        }

        Message::new(
            "UserPotentialSpamUsersQuery",
            json!({ "first": 1, "after": user_id2}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "userIds": [user_id] }));
    }

    #[actix_rt::test]
    async fn fails_when_first_parameter_is_too_high() {
        Message::new("UserPotentialSpamUsersQuery", json!({ "first": 1_000_000 }))
            .execute()
            .await
            .should_be_bad_request();
    }
}

mod user_remove_role_mutation {
    use test_utils::*;

    #[actix_rt::test]
    async fn removes_role_from_user() {
        let user_id: i32 = 10;
        let username = "12297f59";
        let role_name = "sysadmin";
        let role_id: i32 = 11;
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserRemoveRoleMutation",
            json!({ "username": username, "roleName": role_name}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();

        let response = sqlx::query!(
            r#"
                SELECT *
                FROM role_user
                WHERE user_id = ?
                and role_id = ?
            "#,
            user_id,
            role_id
        )
        .fetch_optional(&mut *transaction)
        .await
        .unwrap();

        assert!(response.is_none());
    }

    #[actix_rt::test]
    async fn is_ok_if_user_does_not_have_role() {
        let username = "1229ff10";
        let role_name = "sysadmin";
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserRemoveRoleMutation",
            json!({ "username": username, "roleName": role_name}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok();
    }

    #[actix_rt::test]
    async fn should_throw_bad_request_with_non_existent_role() {
        let username = "admin";
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserRemoveRoleMutation",
            json!({ "username": username, "roleName": "not a role"}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn should_throw_bad_request_with_non_existent_user() {
        let username = "not a user";
        let mut transaction = begin_transaction().await;
        Message::new(
            "UserRemoveRoleMutation",
            json!({ "username": username, "roleName": "login"}),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_bad_request();
    }
}

mod user_by_role_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn should_give_back_first_2_sysadmins_after_id_1() {
        let role_name = "sysadmin";
        let first: i32 = 2;
        let after: i32 = 1;

        Message::new(
            "UsersByRoleQuery",
            json!({"roleName": role_name, "first": first, "after": after}),
        )
        .execute()
        .await
        .should_be_ok_with_body(json!({"usersByRole": [2, 6]}));
    }

    #[actix_rt::test]
    async fn should_give_back_first_2_sysadmins() {
        let role_name = "sysadmin";
        let first: i32 = 2;

        Message::new(
            "UsersByRoleQuery",
            json!({"roleName": role_name, "first": first}),
        )
        .execute()
        .await
        .should_be_ok_with_body(json!({"usersByRole": [1, 2]}));
    }

    #[actix_rt::test]
    async fn should_fail_when_first_is_too_great() {
        let role_name = "sysadmin";
        let first: i32 = 10001;

        Message::new(
            "UsersByRoleQuery",
            json!({"roleName": role_name, "first": first}),
        )
        .execute()
        .await
        .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn should_fail_when_role_does_not_exist() {
        let role_name = "not a role";
        let first: i32 = 10001;

        Message::new(
            "UsersByRoleQuery",
            json!({"roleName": role_name, "first": first}),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

mod user_set_description_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn updates_user_description() {
        let mut transaction = begin_transaction().await;
        let user_id = create_new_test_user(&mut *transaction).await.unwrap();

        Message::new(
            "UserSetDescriptionMutation",
            json!({ "userId": user_id, "description": "new description".to_string() }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "success": true }));

        Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["description"], "new description".to_string())
            });
    }

    #[actix_rt::test]
    async fn fails_when_description_is_longer_than_64kb() {
        Message::new(
            "UserSetDescriptionMutation",
            json!({
                "userId": 1,
                "description": "X".repeat(64*1024)
            }),
        )
        .execute()
        .await
        .should_be_bad_request();
    }
}

mod user_set_email_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn updates_user_email() {
        let mut transaction = begin_transaction().await;
        let user_id = create_new_test_user(&mut *transaction).await.unwrap();
        let new_email = "user@example.com".to_string();
        let user = Message::new("UuidQuery", json!({ "id": user_id }))
            .execute_on(&mut transaction)
            .await
            .get_json();
        let username = user.get("username").unwrap().as_str().unwrap();

        Message::new(
            "UserSetEmailMutation",
            json!({ "userId": user_id, "email": &new_email }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with_body(json!({ "success": true, "username": username }));

        let email = get_email(user_id, &mut *transaction).await.unwrap();

        assert_eq!(email, new_email)
    }
}
