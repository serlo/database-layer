#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use futures::StreamExt;

    use serlo_org_database_layer::event::{EventMessage, EventsQuery};
    use serlo_org_database_layer::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn events_query_without_after_paramater() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let mut app = test::init_service(app).await;
        let message = EventMessage::EventsQuery(EventsQuery {});
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&message)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());

        let (bytes, _) = resp.take_body().into_future().await;
        let events = json::parse(std::str::from_utf8(&bytes.unwrap().unwrap()).unwrap()).unwrap();

        assert_eq!(events["events"].len(), 25_000);
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
        assert_eq!(
            events["events"][10_000],
            json::object! {
                __typename: "CreateEntityNotificationEvent",
                id: 10014,
                instance: "de",
                date: "2014-03-01T21:18:05+01:00",
                actorId: 6,
                objectId: 5545,
                entityId: 5545
            }
        );
    }
}
