#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::str::from_utf8;

    use serlo_org_database_layer::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn events_query_without_after_parameter() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EventsQuery",
                "payload": {}
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let events = json::parse(from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(events["events"].len(), 2_000);
        assert_eq!(
            events["events"][0],
            json::object! {
                __typename: "CreateTaxonomyLinkNotificationEvent",
                id: 1,
                instance: "de",
                date: "2014-03-01T20:36:33+01:00",
                actorId: 6,
                objectId: 8,
                parentId: 8,
                childId: 1199
            }
        );
    }

    #[actix_rt::test]
    async fn events_query_with_after_parameter() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EventsQuery",
                "payload": { "after": 80_013 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let events =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(
            events["events"][0],
            json::object! {
                __typename: "SetLicenseNotificationEvent",
                id: 80014,
                instance: "de",
                date: "2014-10-31T10:54:44+01:00",
                actorId: 324,
                objectId: 32567,
                repositoryId: 32567
            }
        );
    }

    #[actix_rt::test]
    async fn events_query_with_actor_id_parameter() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EventsQuery",
                "payload": { "actorId": 2 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let events =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(events["events"].len(), 13);
        assert_eq!(
            events["events"][0],
            json::object! {
                "__typename": "CreateCommentNotificationEvent",
                "id": 37375,
                "instance": "de",
                "date": "2014-03-01T22:44:29+01:00",
                "actorId": 2,
                "objectId": 15469,
                "threadId": 15468,
                "commentId": 15469
            }
        );
    }

    #[actix_rt::test]
    async fn events_query_with_object_id_parameter() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let app = test::init_service(app).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&serde_json::json!({
                "type": "EventsQuery",
                "payload": { "objectId": 1565 }
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let events =
            json::parse(std::str::from_utf8(&test::read_body(resp).await).unwrap()).unwrap();

        assert_eq!(events["events"].len(), 23);
        assert_eq!(
            events["events"][0],
            json::object! {
                "__typename": "SetLicenseNotificationEvent",
                "id": 472,
                "instance": "de",
                "date": "2014-03-01T20:38:24+01:00",
                "actorId": 6,
                "objectId": 1565,
                "repositoryId": 1565
            }
        );
    }
}
