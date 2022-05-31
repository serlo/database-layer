mod events_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn without_after_parameter() {
        Message::new("EventsQuery", json!({ "first": 100 }))
            .execute()
            .await
            .should_be_ok_with(|result| {
                assert_has_length(&result["events"], 100);
                assert_eq!(result["hasNextPage"], true);
                assert_eq!(
                    result["events"][0],
                    json!({
                        "__typename": "SetTaxonomyTermNotificationEvent",
                        "id": 86591,
                        "instance": "en",
                        "date": "2020-06-16T12:50:13+02:00",
                        "actorId": 1,
                        "objectId": 35607,
                        "taxonomyTermId": 35607
                    })
                );
            });
    }

    #[actix_rt::test]
    async fn with_after_parameter() {
        Message::new("EventsQuery", json!({ "first": 100, "after": 80_015 }))
            .execute()
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["hasNextPage"], true);
                assert_eq!(
                    result["events"][0],
                    json!({
                        "__typename": "SetLicenseNotificationEvent",
                        "id": 80014,
                        "instance": "de",
                        "date": "2014-10-31T10:54:44+01:00",
                        "actorId": 324,
                        "objectId": 32567,
                        "repositoryId": 32567
                    })
                );
            });
    }

    #[actix_rt::test]
    async fn with_actor_id_parameter() {
        Message::new("EventsQuery", json!({ "first": 100, "actorId": 2 }))
            .execute()
            .await
            .should_be_ok_with(|result| {
                assert_has_length(&result["events"], 13);
                assert_eq!(result["hasNextPage"], false);
                assert_eq!(
                    result["events"][12],
                    json!({
                        "__typename": "CreateCommentNotificationEvent",
                        "id": 37375,
                        "instance": "de",
                        "date": "2014-03-01T22:44:29+01:00",
                        "actorId": 2,
                        "objectId": 15469,
                        "threadId": 15468,
                        "commentId": 15469
                    })
                );
            });
    }

    #[actix_rt::test]
    async fn with_object_id_parameter() {
        Message::new("EventsQuery", json!({ "first": 100, "objectId": 1565 }))
            .execute()
            .await
            .should_be_ok_with(|result| {
                assert_has_length(&result["events"], 23);
                assert_eq!(result["hasNextPage"], false);
                assert_eq!(
                    result["events"][22],
                    json!({
                        "__typename": "SetLicenseNotificationEvent",
                        "id": 472,
                        "instance": "de",
                        "date": "2014-03-01T20:38:24+01:00",
                        "actorId": 6,
                        "objectId": 1565,
                        "repositoryId": 1565
                    })
                );
            });
    }

    #[actix_rt::test]
    async fn with_instance_parameter() {
        Message::new("EventsQuery", json!({ "first": 100, "instance": "en" }))
            .execute()
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["hasNextPage"], true);
                assert_eq!(
                    result["events"][0],
                    json!({
                        "__typename": "SetTaxonomyTermNotificationEvent",
                        "id": 86591,
                        "instance": "en",
                        "date": "2020-06-16T12:50:13+02:00",
                        "actorId": 1,
                        "objectId": 35607,
                        "taxonomyTermId": 35607
                    })
                );
            });
    }

    #[actix_rt::test]
    async fn fails_when_first_parameter_is_too_high() {
        Message::new("EventsQuery", json!({ "first": 1_000_000 }))
            .execute()
            .await
            .should_be_bad_request();
    }
}
