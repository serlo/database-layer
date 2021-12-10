#[cfg(test)]
mod entities_metadata_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_metadata_of_entities() {
        let response = Message::new("EntitiesMetadataQuery", json!({ "first": 1 }))
            .execute()
            .await;

        assert_ok(
            response,
            json!({
              "entities": [
                {
                  "@context": [
                    "https://w3id.org/kim/lrmi-profile/draft/context.jsonld",
                    {
                      "@language": "de"
                    }
                  ],
                  "id": "https://serlo.org/1495",
                  "identifier": {
                    "type": "PropertyValue",
                    "propertyID": "UUID",
                    "value": 1495
                  },
                  "type": [
                    "LearningResource",
                    "Article"
                  ],
                  "learningResourceType": "Article",
                  "name": "Addition",
                  "description": null,
                  "dateCreated": "2014-03-01T20:36:44+00:00",
                  "dateModified": "2014-10-31T15:56:50+00:00",
                  "publisher": "https://serlo.org/",
                  "maintainer": "https://serlo.org/",
                  "license": {
                    "id": "https://creativecommons.org/licenses/by-sa/4.0/"
                  },
                  "version": "https://serlo.org/32614"
                }
              ]
            }),
        )
        .await;
    }

    #[actix_rt::test]
    async fn with_after_parameter() {
        let response = Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 1945 }),
        )
        .execute()
        .await;

        assert_ok_with(response, |value| {
            assert_eq!(value["entities"][0]["identifier"]["value"], 1947)
        })
        .await;
    }

    #[actix_rt::test]
    async fn with_instance_parameter() {
        let response = Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "instance": "en" }),
        )
        .execute()
        .await;

        assert_ok_with(response, |value| {
            assert_eq!(value["entities"][0]["identifier"]["value"], 32996)
        })
        .await;
    }

    #[actix_rt::test]
    async fn with_modified_after_parameter() {
        let response = Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "modifiedAfter": "2015-01-01T00:00:00Z" }),
        )
        .execute()
        .await;

        assert_ok_with(response, |value| {
            assert_eq!(value["entities"][0]["identifier"]["value"], 1647)
        })
        .await;
    }

    #[actix_rt::test]
    async fn fails_when_first_parameter_is_too_high() {
        let response = Message::new("EntitiesMetadataQuery", json!({ "first": 1_000_000 }))
            .execute()
            .await;

        assert_bad_request(response, "The 'first' value should be less than 10_000").await;
    }
}