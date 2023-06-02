mod entities_metadata_query {
    use chrono::{DateTime, Duration, Utc};
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};
    use test_utils::{assert_eq, *};

    #[actix_rt::test]
    async fn returns_metadata_for_articles() {
        assert_metadata(json!({
          "@context": [
            "https://w3id.org/kim/amb/context.jsonld",
            {
              "@language": "de",
              "@vocab": "http://schema.org/",
              "type": "@type",
              "id": "@id"
            }
          ],
          "id": "https://serlo.org/1495",
          "type": [ "LearningResource", "Article" ],
          "creator": [
            {
              "id": "https://serlo.org/324",
              "name": "122d486a",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
            {
              "id": "https://serlo.org/15491",
              "name": "125f4a84",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
            {
              "id": "https://serlo.org/22573",
              "name": "12600e93",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
            {
              "id": "https://serlo.org/1",
              "name": "admin",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
            {
              "id": "https://serlo.org/6",
              "name": "12297c72",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
            {
              "id": "https://serlo.org/677",
              "name": "124902c9",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
            {
              "id": "https://serlo.org/15473",
              "name": "125f3e12",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
            {
              "id": "https://serlo.org/15478",
              "name": "125f467c",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },

            {
              "id": "https://serlo.org/27689",
              "name": "1268a3e2",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              }
            },
          ],
          "dateCreated": "2014-03-01T20:36:44+00:00",
          "dateModified": "2014-10-31T15:56:50+00:00",
          "headline": "Addition",
          "identifier": {
            "type": "PropertyValue",
            "propertyID": "UUID",
            "value": 1495
          },
          "isAccessibleForFree": true,
          "isFamilyFriendly": true,
          "inLanguage": [ "de" ],
          "learningResourceType": [
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/text" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/worksheet" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/course" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/web_page" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/wiki" },
          ],
          "license": { "id": "https://creativecommons.org/licenses/by-sa/4.0/" },
          "mainEntityOfPage": [{
            "id": "https://serlo.org/metadata-api",
            "provider": {
               "id": "https://serlo.org/organization",
               "type": "Organization",
               "name": "Serlo Education e.V."
            },
          }],
          "maintainer": {
            "id": "https://serlo.org/organization",
            "type": "Organization",
            "name": "Serlo Education e.V.",
          },
          "name": "Addition",
          "isPartOf": [
            { "id": "https://serlo.org/1292" },
            { "id": "https://serlo.org/16072" },
            { "id": "https://serlo.org/16174" },
            { "id": "https://serlo.org/33119" },
            { "id": "https://serlo.org/34743" },
            { "id": "https://serlo.org/34744" },
          ],
          "publisher": [
            {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V.",
            }
          ],
          "version": "https://serlo.org/32614"
        }))
        .await;
    }

    #[actix_rt::test]
    async fn returns_metadata_for_applets() {
        assert_metadata(json!({
          "@context": [
            "https://w3id.org/kim/amb/context.jsonld",
            {
              "@language": "en",
              "@vocab": "http://schema.org/",
              "type": "@type",
              "id": "@id"
            }
          ],
          "id": "https://serlo.org/35596",
          "type": [ "LearningResource", "WebApplication" ],
          "creator": [
            {
              "id": "https://serlo.org/1",
              "name": "admin",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            },
          ],
          "dateCreated": "2020-01-29T17:47:19+00:00",
          "dateModified": "2020-01-29T17:48:54+00:00",
          "headline": "Example applet",
          "identifier": {
            "propertyID": "UUID",
            "type": "PropertyValue",
            "value": 35596
          },
          "inLanguage": [ "en" ],
          "isAccessibleForFree": true,
          "isFamilyFriendly": true,
          "learningResourceType": [
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/application" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/demonstration" },
          ],
          "license": { "id": "http://creativecommons.org/licenses/by/4.0/" },
          "mainEntityOfPage": [{
            "id": "https://serlo.org/metadata-api",
            "provider": {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V."
            },
          }],
          "maintainer": {
            "id": "https://serlo.org/organization",
            "type": "Organization",
            "name": "Serlo Education e.V.",
          },
          "name": "Example applet",
          "publisher": [
            {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V.",
            },
          ],
          "isPartOf": [{ "id": "https://serlo.org/35560" }],
          "version": "https://serlo.org/35597"
        }))
        .await;
    }

    #[actix_rt::test]
    async fn returns_metadata_for_courses() {
        assert_metadata(json!({
          "@context": [
            "https://w3id.org/kim/amb/context.jsonld",
            {
              "@language": "de",
              "@vocab": "http://schema.org/",
              "type": "@type",
              "id": "@id"
            }
          ],
          "id": "https://serlo.org/18514",
          "type": [ "LearningResource", "Course" ],
          "creator": [
            {
              "id": "https://serlo.org/324",
              "name": "122d486a",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            },
            {
              "id": "https://serlo.org/1",
              "name": "admin",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            },
          ],
          "dateCreated": "2014-03-17T12:22:17+00:00",
          "dateModified": "2014-09-16T07:47:55+00:00",
          "headline": "Überblick zum Satz des Pythagoras",
          "identifier": {
            "propertyID": "UUID",
            "type": "PropertyValue",
            "value": 18514
          },
          "inLanguage": [ "de" ],
          "isAccessibleForFree": true,
          "isFamilyFriendly": true,
          "learningResourceType": [
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/course" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/exploration" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/web_page" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/wiki" },
          ],
          "license": {
            "id": "https://creativecommons.org/licenses/by-sa/4.0/"
          },
          "mainEntityOfPage": [{
            "id": "https://serlo.org/metadata-api",
            "provider": {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V."
            },
          }],
          "maintainer": {
            "id": "https://serlo.org/organization",
            "type": "Organization",
            "name": "Serlo Education e.V.",
          },
          "name": "Überblick zum Satz des Pythagoras",
          "isPartOf": [
            { "id": "https://serlo.org/1381" },
            { "id": "https://serlo.org/16526" },
          ],
          "publisher": [
            {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V.",
            },
          ],
          "version": "https://serlo.org/30713"
        }))
        .await;
    }

    #[actix_rt::test]
    async fn returns_metadata_for_exercises() {
        assert_metadata(json!({
          "@context": [
            "https://w3id.org/kim/amb/context.jsonld",
            {
              "@language": "de",
              "@vocab": "http://schema.org/",
              "type": "@type",
              "id": "@id"
            }
          ],
          "id": "https://serlo.org/2823",
          "type": [ "LearningResource", "Quiz" ],
          "creator": [
            {
              "id": "https://serlo.org/6",
              "name": "12297c72",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            },
          ],
          "dateCreated": "2014-03-01T21:02:56+00:00",
          "dateModified": "2014-03-01T21:02:56+00:00",
          "identifier": {
            "propertyID": "UUID",
            "type": "PropertyValue",
            "value": 2823
          },
          "inLanguage": [ "de" ],
          "isAccessibleForFree": true,
          "isFamilyFriendly": true,
          "isPartOf": [{ "id": "https://serlo.org/25614" }],
          "learningResourceType": [
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/drill_and_practice" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/assessment" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/web_page" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/wiki" }
          ],
          "license": { "id": "https://creativecommons.org/licenses/by-sa/4.0/" },
          "mainEntityOfPage": [{
            "id": "https://serlo.org/metadata-api",
            "provider": {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V."
            },
          }],
          "maintainer": {
            "id": "https://serlo.org/organization",
            "type": "Organization",
            "name": "Serlo Education e.V.",
          },
          "name": "Aufgabe aus \"Aufgaben zum Thema Ergebnisraum oder Ergebnismenge\"",
          "publisher": [
            {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V.",
            },
          ],
          "version": "https://serlo.org/2824"
        }))
        .await;
    }

    #[actix_rt::test]
    async fn returns_metadata_for_exercise_groups() {
        assert_metadata(json!({
          "@context": [
            "https://w3id.org/kim/amb/context.jsonld",
            {
              "@language": "de",
              "@vocab": "http://schema.org/",
              "type": "@type",
              "id": "@id"
            }
          ],
          "id": "https://serlo.org/2217",
          "type": [ "LearningResource", "Quiz" ],
          "creator": [
            {
              "id": "https://serlo.org/6",
              "name": "12297c72",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            },
          ],
          "dateCreated": "2014-03-01T20:54:51+00:00",
          "dateModified": "2014-03-01T20:54:51+00:00",
          "identifier": {
            "propertyID": "UUID",
            "type": "PropertyValue",
            "value": 2217
          },
          "inLanguage": [ "de" ],
          "isAccessibleForFree": true,
          "isFamilyFriendly": true,
          "learningResourceType": [
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/drill_and_practice" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/assessment" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/web_page" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/wiki" },
          ],
          "license": { "id": "https://creativecommons.org/licenses/by-sa/4.0/" },
          "mainEntityOfPage": [{
            "id": "https://serlo.org/metadata-api",
            "provider": {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V."
            },
          }],
          "maintainer": {
            "id": "https://serlo.org/organization",
            "type": "Organization",
            "name": "Serlo Education e.V.",
          },
          "name": "Aufgabe aus \"Sachaufgaben\"",
          "isPartOf": [
            { "id": "https://serlo.org/21804" },
            { "id": "https://serlo.org/25726" },
          ],
          "publisher": [
            {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V.",
            },
          ],
          "version": "https://serlo.org/2218"
        }))
        .await;
    }

    #[actix_rt::test]
    async fn returns_metadata_for_videos() {
        assert_metadata(json!({
          "@context": [
            "https://w3id.org/kim/amb/context.jsonld",
            {
              "@language": "de",
              "@vocab": "http://schema.org/",
              "type": "@type",
              "id": "@id"
            }
          ],
          "id": "https://serlo.org/18865",
          "type": [ "LearningResource", "Video" ],
          "creator": [
            {
              "id": "https://serlo.org/22573",
              "name": "12600e93",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            },
            {
              "id": "https://serlo.org/15478",
              "name": "125f467c",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            },
            {
              "id": "https://serlo.org/15491",
              "name": "125f4a84",
              "type": "Person",
              "affiliation": {
                "id": "https://serlo.org/organization",
                "type": "Organization",
                "name": "Serlo Education e.V.",
              },
            }
          ],
          "dateCreated": "2014-03-17T16:18:44+00:00",
          "dateModified": "2014-05-01T09:22:14+00:00",
          "headline": "Satz des Pythagoras",
          "identifier": {
            "propertyID": "UUID",
            "type": "PropertyValue",
            "value": 18865
          },
          "inLanguage": [ "de" ],
          "isAccessibleForFree": true,
          "isFamilyFriendly": true,
          "learningResourceType": [
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/video" },
            { "id": "http://w3id.org/openeduhub/vocabs/learningResourceType/audiovisual_medium" },
          ],
          "license": { "id": "https://creativecommons.org/licenses/by-sa/4.0/" },
          "mainEntityOfPage": [{
            "id": "https://serlo.org/metadata-api",
            "provider": {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V."
            },
          }],
          "maintainer": {
            "id": "https://serlo.org/organization",
            "type": "Organization",
            "name": "Serlo Education e.V.",
          },
          "name": "Satz des Pythagoras",
          "isPartOf": [
            { "id": "https://serlo.org/1381" },
            { "id": "https://serlo.org/16214" },
          ],
          "publisher": [
            {
              "id": "https://serlo.org/organization",
              "type": "Organization",
              "name": "Serlo Education e.V.",
            },
          ],
          "version": "https://serlo.org/24383"
        }))
        .await;
    }

    #[actix_rt::test]
    async fn shows_description_if_not_empty_nor_null() {
        let mut transaction = begin_transaction().await;

        sqlx::query!(
            r#"
            update entity_revision_field set value = "description for entity 2153"
            where id = 41509 and field = "meta_description";
        "#
        )
        .execute(&mut transaction)
        .await
        .unwrap();

        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 2152 }),
        )
        .execute_on(&mut transaction)
        .await
        .should_be_ok_with(|result| {
            assert_json_include!(
              actual: &result["entities"][0],
              expected: json!({
                  "id": "https://serlo.org/2153",
                  "description": "description for entity 2153"
              })
            )
        });
    }

    #[actix_rt::test]
    async fn returns_metadata_for_entities_and_sorts_creators_on_edit_count() {
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 9066 }),
        )
        .execute()
        .await
        .should_be_ok_with(|result| {
            let creator_urls: Vec<&str> = result["entities"][0]["creator"]
                .as_array()
                .map(|array| array.into_iter().filter_map(|x| x["id"].as_str()).collect())
                .unwrap();

            // There are two edits from user with id 15491 which is why they
            // should be listed first
            assert_eq!(
                creator_urls,
                vec!["https://serlo.org/15491", "https://serlo.org/6"]
            );
        });
    }

    #[actix_rt::test]
    async fn assert_query_is_faster_than_3000ms() {
        let start = now();

        Message::new("EntitiesMetadataQuery", json!({ "first": 10_000 }))
            .execute()
            .await
            .should_be_ok();

        let duration = now() - start;

        // Querying 10.000 elements should be faster than 3 seconds, so that querying all entities
        // will take less than 30 seconds (At April 2023 we had ~50.000 entities so even if we add
        // taxonomies in the future it will be less than 100.000 objects).
        assert!(duration < 3000, "Duration of {:}ms is too high", duration);
    }

    #[actix_rt::test]
    async fn with_after_parameter() {
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 1945 }),
        )
        .execute()
        .await
        .should_be_ok_with(|value| assert_eq!(value["entities"][0]["identifier"]["value"], 1947));
    }

    #[actix_rt::test]
    async fn with_instance_parameter() {
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "instance": "en" }),
        )
        .execute()
        .await
        .should_be_ok_with(|value| assert_eq!(value["entities"][0]["identifier"]["value"], 32996));
    }

    #[actix_rt::test]
    async fn with_modified_after_parameter() {
        env::set_var("METADATA_API_LAST_CHANGES_DATE", "2014-12-30T00:00:00Z");
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "modifiedAfter": "2015-01-01T00:00:00Z" }),
        )
        .execute()
        .await
        .should_be_ok_with(|value| assert_eq!(value["entities"][0]["identifier"]["value"], 1647));

        env::set_var("METADATA_API_LAST_CHANGES_DATE", "2015-01-01T00:00:01Z");
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "modifiedAfter": "2015-01-01T00:00:00Z" }),
        )
        .execute()
        .await
        .should_be_ok_with(|value| assert_eq!(value["entities"][0]["identifier"]["value"], 1495));
    }

    #[actix_rt::test]
    async fn fails_when_first_parameter_is_too_high() {
        Message::new("EntitiesMetadataQuery", json!({ "first": 1_000_000 }))
            .execute()
            .await
            .should_be_bad_request();
    }

    fn now() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    async fn assert_metadata(mut expected_metadata: Value) {
        let id = expected_metadata["identifier"]["value"].as_u64().unwrap();

        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": id - 1 }),
        )
        .execute()
        .await
        .should_be_ok_with(|actual_response: Value| {
            let parse_date = |property: &str| {
                DateTime::parse_from_rfc3339(
                    &actual_response["entities"][0]["mainEntityOfPage"][0][property]
                        .as_str()
                        .unwrap(),
                )
                .unwrap()
            };
            let actual_date_created = parse_date("dateCreated");
            let actual_date_modified = parse_date("dateModified");

            assert!(Utc::now() < actual_date_created + Duration::seconds(1));
            assert!(Utc::now() < actual_date_modified + Duration::seconds(1));

            expected_metadata["mainEntityOfPage"][0]["dateCreated"] = json!(actual_date_created);
            expected_metadata["mainEntityOfPage"][0]["dateModified"] = json!(actual_date_modified);

            let expected_response = json!({ "entities": [expected_metadata] });

            assert_eq!(expected_response, actual_response);
        });
    }
}
