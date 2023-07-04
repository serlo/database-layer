#![recursion_limit = "256"]

mod add_revision_mutation {
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn adds_revision() {
        let mut transaction = begin_transaction().await;

        let new_revision_id = Message::new(
            "PageAddRevisionMutation",
            json!({
                "pageId": 16256,
                "content": "test content",
                "title": "test title",
                "userId": 1,
            }),
        )
        .execute_on(&mut *transaction)
        .await
        .get_json()["revisionId"]
            .clone();

        Message::new("UuidQuery", json!({ "id": new_revision_id }))
            .execute_on(&mut *transaction)
            .await
            .should_be_ok_with(|result| {
                assert_eq!(result["content"], "test content");
                assert_eq!(result["title"], "test title");
                assert_eq!(result["authorId"], 1 as i32);
            });
    }
}

mod create_mutation {
    use serde_json::Value::Null;
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn creates_page() {
        Message::new(
            "PageCreateMutation",
            json!({
                "content": "test content",
                "discussionsEnabled": false,
                "forumId": Null,
                "instance": "de",
                "licenseId": 1 as i32,
                "title": "test title",
                "userId": 1 as i32,
            }),
        )
        .execute()
        .await
        .should_be_ok_with(|result| {
            assert_eq!(result["instance"], "de");
            assert_eq!(result["licenseId"], 1 as i32);
        })
    }
}

mod pages_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn fetches_all_pages() {
        Message::new("PagesQuery", json!({}))
            .execute()
            .await
            .should_be_ok_with_body(json!({
                "pages": [
                    16256,16303,16306,16530,16569,16659,16816,18233,18340,18778,18922,18998,19358,
                    19722,19723,19757,19763,19767,19808,19849,19852,19854,19856,19860,19863,19865,
                    19869,19871,19875,19880,19882,19973,19981,19991,19996,20003,20064,20076,20103,
                    20112,20114,20125,20136,20182,20205,20307,21160,21163,21398,21406,21408,21413,
                    21421,21423,21427,21429,21431,21433,21435,21437,21439,21456,21468,21470,21472,
                    21475,21511,21526,21538,21541,21543,21549,21551,21553,21555,21557,21559,21561,
                    21563,21565,21567,21570,21654,21657,21933,22886,22964,23111,23320,23439,23534,
                    23576,23579,23580,23591,23711,23720,23727,23950,24214,24706,24711,24806,24887,
                    25017,25019,25063,25079,25082,25294,25363,25373,25598,25713,25985,26087,26089,
                    26095,26245,26453,26473,26524,26542,26544,26546,26592,26633,26639,26874,26880,
                    27203,27421,27469,27472,31996,32840,32875,32966,35093,35096,35098,35100,35152
                ]
            }))
    }

    #[actix_rt::test]
    async fn fetches_all_pages_of_instance() {
        Message::new("PagesQuery", json!({"instance": "en"}))
            .execute()
            .await
            .should_be_ok_with_body(json!({
                "pages": [23579, 23580, 23591, 23711, 23720, 23727, 25079, 25082, 27469, 32840, 32966]
            }))
    }

    #[actix_rt::test]
    async fn fetches_empty_set() {
        Message::new("PagesQuery", json!({"instance": "hi"}))
            .execute()
            .await
            .should_be_ok_with_body(json!({ "pages": [] }))
    }
}
