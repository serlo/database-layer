mod entities_metadata_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_metadata_for_articles() {
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
    async fn returns_metadata_for_applets() {
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 35595 }),
        )
        .execute()
        .await
        .should_be_ok_with_body(json!({
            "entities": [
              {
                "@context": [
                  "https://w3id.org/kim/lrmi-profile/draft/context.jsonld",
                  { "@language": "en" }
                ],
                "id": "https://serlo.org/35596",
                "type": [
                  "LearningResource",
                  ""
                ],
                "dateCreated": "2020-01-29T17:47:19+00:00",
                "dateModified": "2020-01-29T17:48:54+00:00",
                "description": "",
                "headline": "Example applet",
                "identifier": {
                  "propertyID": "UUID",
                  "type": "PropertyValue",
                  "value": 35596
                },
                "inLanguage": [ "en" ],
                "isAccessibleForFree": true,
                "isFamilyFriendly": true,
                "learningResourceType": "",
                "license": { "id": "http://creativecommons.org/licenses/by/4.0/" },
                "maintainer": "https://serlo.org/",
                "name": "Example applet",
                "publisher": [{ "id": "https://serlo.org/" }],
                "version": "https://serlo.org/35597"
              }
            ]
        }));
    }

    #[actix_rt::test]
    async fn returns_metadata_for_courses() {
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 18274 }),
        )
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
              "id": "https://serlo.org/18514",
              "type": [
                "LearningResource",
                "Course"
              ],
              "dateCreated": "2014-03-17T12:22:17+00:00",
              "dateModified": "2014-09-16T07:47:55+00:00",
              "description": null,
              "headline": "Überblick zum Satz des Pythagoras",
              "identifier": {
                "propertyID": "UUID",
                "type": "PropertyValue",
                "value": 18514
              },
              "inLanguage": [
                "de"
              ],
              "isAccessibleForFree": true,
              "isFamilyFriendly": true,
              "learningResourceType": "Course",
              "license": {
                "id": "https://creativecommons.org/licenses/by-sa/4.0/"
              },
              "maintainer": "https://serlo.org/",
              "name": "Überblick zum Satz des Pythagoras",
              "publisher": [
                {
                  "id": "https://serlo.org/"
                }
              ],
              "version": "https://serlo.org/30713"
            }
          ]
        }));
    }

    #[actix_rt::test]
    async fn returns_metadata_for_exercises() {
        Message::new(
            "EntitiesMetadataQuery",
            json!({ "first": 1, "after": 2327 }),
        )
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
              "id": "https://serlo.org/2331",
              "type": [
                "LearningResource",
                "Quiz"
              ],
              "dateCreated": "2014-03-01T20:55:29+00:00",
              "dateModified": "2014-03-10T14:33:05+00:00",
              "description": null,
              "headline": null,
              "identifier": {
                "propertyID": "UUID",
                "type": "PropertyValue",
                "value": 2331
              },
              "inLanguage": [
                "de"
              ],
              "isAccessibleForFree": true,
              "isFamilyFriendly": true,
              "learningResourceType": "Quiz",
              "license": {
                "id": "https://creativecommons.org/licenses/by-sa/4.0/"
              },
              "maintainer": "https://serlo.org/",
              "name": "Quiz: https://serlo.org/2331",
              "publisher": [
                {
                  "id": "https://serlo.org/"
                }
              ],
              "version": "https://serlo.org/16573"
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
