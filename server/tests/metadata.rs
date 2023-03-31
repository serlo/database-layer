mod entities_metadata_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_metadata_of_entities() {
        Message::new("EntitiesMetadataQuery", json!({ "first": 1 }))
            .execute()
            .await
            .should_be_ok_with_body(json!({
              "entities": [
                {
                  "@context": [
                    "https://w3id.org/kim/lrmi-profile/draft/context.jsonld",
                    {
                      "@language": "de"
                    }
                  ],
                  "id": "https://serlo.org/1495",
                  "type": [
                    "LearningResource",
                    "Article"
                  ],
                  "dateCreated": "2014-03-01T20:36:44+00:00",
                  "dateModified": "2014-10-31T15:56:50+00:00",
                  "description": null,
                  "headline": "Addition",
                  "identifier": {
                    "type": "PropertyValue",
                    "propertyID": "UUID",
                    "value": 1495
                  },
                  "isAccessibleForFree": true,
                  "isFamilyFriendly": true,
                  "inLanguage": [ "de" ],
                  "learningResourceType": "Article",
                  "license": {
                    "id": "https://creativecommons.org/licenses/by-sa/4.0/"
                  },
                  "maintainer": "https://serlo.org/",
                  "name": "Addition",
                  "publisher": [{
                      "id": "https://serlo.org/"
                  }],
                  "version": "https://serlo.org/32614"
                }
              ]
            }));
    }

    #[actix_rt::test]
    async fn default_value_for_property_name() {
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 20_000 }),
        )
        .execute()
        .await
        .should_be_ok_with(|value| {
            assert_eq!(
                value["entities"][0]["name"],
                "Quiz: https://serlo.org/20256"
            )
        });
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
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "modifiedAfter": "2015-01-01T00:00:00Z" }),
        )
        .execute()
        .await
        .should_be_ok_with(|value| assert_eq!(value["entities"][0]["identifier"]["value"], 1647));
    }

    #[actix_rt::test]
    async fn fails_when_first_parameter_is_too_high() {
        Message::new("EntitiesMetadataQuery", json!({ "first": 1_000_000 }))
            .execute()
            .await
            .should_be_bad_request();
    }

    #[actix_rt::test]
    async fn checks_mandatory_keys_in_response() {
        Message::new("EntitiesMetadataQuery", json!({ "first": 350 }))
            .execute()
            .await
            .should_be_ok_with(|value| {
                let mandatory_keys = [
                    "@context",
                    "id",
                    "type",
                    "name",
                    "publisher",
                    "learningResourceType",
                ];

                let entities = value["entities"].as_array().unwrap();
                for entity in entities {
                    for mandatory_key in &mandatory_keys {
                        assert!(
                            entity.get(mandatory_key).is_some(),
                            "Mandatory key '{}' is missing from the response",
                            mandatory_key
                        );
                    }
                }
            });
    }

    #[actix_rt::test]
    async fn checks_mandatory_keys_in_response_of_a_quiz() {
        // Fetching a quiz https://serlo.org/20256
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 20_000  }),
        )
        .execute()
        .await
        .should_be_ok_with(|value| {
            assert_eq!(
                value["entities"][0]["name"],
                "Quiz: https://serlo.org/20256"
            );
            let mandatory_keys = [
                "@context",
                "id",
                "type",
                "name",
                "publisher",
                "learningResourceType",
            ];

            let entities = value["entities"].as_array().unwrap();
            for entity in entities {
                for mandatory_key in &mandatory_keys {
                    assert!(
                        entity.get(mandatory_key).is_some(),
                        "Mandatory key '{}' is missing from the Quiz!",
                        mandatory_key
                    );
                }
            }
        });
    }

    #[actix_rt::test]
    async fn checks_optional_keys_in_response() {
        Message::new("EntitiesMetadataQuery", json!({ "first": 300, }))
            .execute()
            .await
            .should_be_ok_with(|value| {
                let optional_keys = [
                    "description",
                    "about",
                    "keywords",
                    "inLanguage",
                    "image",
                    "trailer",
                    "creator",
                    "contributor",
                    "affiliation",
                    "dateCreated",
                    "datePublished",
                    "dateModified",
                    "isAccessibleForFree",
                    "license",
                    "conditionsOfAccess",
                    "audience",
                    "teaches",
                    "assesses",
                    "competencyRequired",
                    "educationalLevel",
                    "interactivityType",
                    "isBasedOn",
                    "isPartOf",
                    "hasPart",
                    "mainEntityOfPage",
                    "duration",
                    "encoding",
                    "caption",
                ];

                let entities = value["entities"].as_array().unwrap();
                for entity in entities {
                    for optional_key in &optional_keys {
                        if let Some(field_value) = entity.get(optional_key) {
                            // TODO add more specific checks for each optional
                            // field's value and their respective schemas
                            match *optional_key {
                                "isAccessibleForFree" => {
                                    assert!(
                                        field_value.as_bool() == Some(true),
                                        "Invalid value for optional key '{}'.
                                        We only offer freely accessible learning resources!",
                                        optional_key
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            });
    }
}
