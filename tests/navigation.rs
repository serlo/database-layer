#[cfg(test)]
mod tests {
    use actix_web::web::BytesMut;
    use actix_web::{test, App};
    use futures::StreamExt;

    use serlo_org_database_layer::{configure_app, create_database_pool};

    #[actix_rt::test]
    async fn de() {
        let pool = create_database_pool().await.unwrap();
        let app = configure_app(App::new(), pool);
        let mut app = test::init_service(app).await;
        let request = test::TestRequest::get().uri("/navigation/de").to_request();
        let mut response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());

        let mut body = response.take_body();
        let mut bytes = BytesMut::new();
        while let Some(item) = body.next().await {
            bytes.extend_from_slice(&item.unwrap());
        }

        let actual = json::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
        let expected = json::parse(
            r#"
                {
                  "data": [
                    {
                      "children": [
                        {
                          "label": "Mathe Startseite",
                          "id": 19767
                        },
                        {
                          "label": "Thema auswählen",
                          "id": 5
                        },
                        {
                          "label": "Nach Lehrplan (Gymnasium Bayern)",
                          "id": 16042
                        },
                        {
                          "children": [
                            {
                              "label": "Neu hier?",
                              "id": 19852
                            },
                            {
                              "label": "Aktuelles und Planung",
                              "id": 19880
                            },
                            {
                              "label": "Übersicht aller Diskussionen",
                              "url": "/discussions/15465"
                            },
                            {
                              "label": "Arbeitsgruppen",
                              "id": 26639
                            },
                            {
                              "label": "Zuständigkeiten",
                              "id": 19763
                            },
                            {
                              "label": "Richtlinien",
                              "id": 19723
                            },
                            {
                              "label": "Neue Bearbeitungen",
                              "url": "/mathe/entity/trash-bin"
                            },
                            {
                              "label": "Lehrplan-Struktur im Aufbau",
                              "id": 5
                            },
                            {
                              "label": "Taxonomy bearbeiten",
                              "url": "/taxonomy/term/organize/5"
                            },
                            {
                              "label": "Papierkorb",
                              "url": "/mathe/entity/trash-bin"
                            }
                          ],
                          "label": "Mathe Community"
                        }
                      ],
                      "label": "Mathematik",
                      "id": 5
                    },
                    {
                      "children": [
                        {
                          "label": "Permakultur Startseite",
                          "id": 24711
                        },
                        {
                          "label": "Thema auswählen",
                          "id": 17744
                        },
                        {
                          "children": [
                            {
                              "label": "Neu hier?",
                              "id": 25294
                            },
                            {
                              "label": "Aktuelles und Planung",
                              "id": 21541
                            },
                            {
                              "label": "Übersicht aller Diskussionen",
                              "url": "/discussions/17746"
                            },
                            {
                              "children": [
                                {
                                  "label": "Themenbaum bearbeiten",
                                  "id": 25373
                                }
                              ],
                              "label": "Arbeitsgruppen"
                            },
                            {
                              "label": "Zuständigkeiten",
                              "id": 21543
                            },
                            {
                              "label": "Richtlinien",
                              "id": 25363
                            },
                            {
                              "label": "Neue Bearbeitungen",
                              "url": "/permakultur/entity/trash-bin"
                            },
                            {
                              "label": "Struktur bearbeiten",
                              "url": "/taxonomy/term/organize/17744"
                            }
                          ],
                          "label": "Permakultur Community"
                        }
                      ],
                      "label": "Permakultur",
                      "id": 17744
                    },
                    {
                      "children": [
                        {
                          "label": "Chemie Startseite",
                          "id": 24706
                        },
                        {
                          "label": "Thema auswählen",
                          "id": 18230
                        },
                        {
                          "children": [
                            {
                              "label": "Neu hier?",
                              "id": 26633
                            },
                            {
                              "label": "Aktuelles und Planung",
                              "id": 31996
                            },
                            {
                              "label": "Neue Bearbeitungen",
                              "url": "/Chemie/entity/trash-bin"
                            },
                            {
                              "label": "Übersicht aller Diskussionen",
                              "url": "/discussions/18234"
                            },
                            {
                              "label": "Richtlinien",
                              "id": 26087
                            },
                            {
                              "label": "Taxonomy bearbeiten",
                              "url": "/taxonomy/term/organize/18230"
                            }
                          ],
                          "label": "Chemie Community"
                        }
                      ],
                      "label": "Chemie",
                      "id": 18230
                    },
                    {
                      "children": [
                        {
                          "label": "Auf einen Blick",
                          "id": 18922
                        },
                        {
                          "children": [
                            {
                              "label": "Grundprinzipien",
                              "id": 21408
                            },
                            {
                              "label": "Vision und Werte",
                              "id": 21398
                            },
                            {
                              "label": "Wirkung",
                              "id": 21406
                            },
                            {
                              "label": "Die Geschichte von Serlo",
                              "id": 21413
                            },
                            {
                              "label": "Eigene Softwareentwicklung",
                              "id": 21431
                            }
                          ],
                          "label": "Was Serlo ausmacht"
                        },
                        {
                          "children": [
                            {
                              "label": "Didaktisches Konzept",
                              "id": 21423
                            },
                            {
                              "label": "Qualitätssicherung",
                              "id": 21429
                            },
                            {
                              "label": "Kooperation mit Schulen",
                              "id": 21433
                            }
                          ],
                          "label": "Lernen und Qualität"
                        },
                        {
                          "children": [
                            {
                              "label": "Übersicht",
                              "id": 21468
                            },
                            {
                              "label": "Nutzungsstatistiken",
                              "id": 23534
                            },
                            {
                              "label": "Entscheidungsfindung",
                              "id": 21470
                            },
                            {
                              "label": "Jahresberichte und Finanzen",
                              "id": 21472
                            }
                          ],
                          "label": "Transparenz"
                        },
                        {
                          "label": "Menschen",
                          "id": 21439
                        },
                        {
                          "label": "Partner und Förderer",
                          "id": 21456
                        },
                        {
                          "label": "Trägerverein",
                          "id": 21437
                        },
                        {
                          "label": "Kontakt",
                          "id": 21657
                        }
                      ],
                      "label": "Über Serlo"
                    },
                    {
                      "children": [
                        {
                          "label": "Community Startseite",
                          "id": 19882
                        },
                        {
                          "label": "Verhaltenskodex",
                          "id": 19875
                        },
                        {
                          "label": "Portal der Richtlinien",
                          "id": 20076
                        },
                        {
                          "label": "Portal der Hilfeseiten",
                          "id": 20064
                        },
                        {
                          "children": [
                            {
                              "label": "Fächerübergreifende Zuständigkeiten",
                              "id": 21570
                            },
                            {
                              "label": "Vergabe aller Zuständigkeiten",
                              "id": 19856
                            }
                          ],
                          "label": "Zuständigkeiten"
                        },
                        {
                          "label": "Alle Aktivitäten auf Serlo",
                          "url": "/event/history"
                        },
                        {
                          "label": "Übersicht aller Diskussionen",
                          "url": "/discussions"
                        },
                        {
                          "children": [
                            {
                              "label": "Moderation",
                              "id": 20112
                            },
                            {
                              "label": "Vertretung der Community",
                              "id": 20114
                            },
                            {
                              "label": "Verwaltung der DE Sprachversion",
                              "id": 20103
                            },
                            {
                              "label": "Neue Fächer",
                              "id": 19863
                            },
                            {
                              "label": "Horizont",
                              "id": 18340
                            },
                            {
                              "label": "Blog schreiben",
                              "id": 20125
                            },
                            {
                              "children": [
                                {
                                  "label": "Ausbildungen auf Serlo",
                                  "id": 19865
                                },
                                {
                                  "label": "Lernbausteine",
                                  "id": 20205
                                }
                              ],
                              "label": "Besondere Projekte"
                            },
                            {
                              "label": "Lizenzen verwalten",
                              "id": 20307
                            },
                            {
                              "label": "Internationalisierung",
                              "id": 24214
                            },
                            {
                              "label": "International system administration",
                              "id": 20182
                            },
                            {
                              "children": [
                                {
                                  "label": "development process",
                                  "id": 21160
                                },
                                {
                                  "label": "Feature suggestions",
                                  "id": 21163
                                }
                              ],
                              "label": "Software development"
                            }
                          ],
                          "label": "Arbeitsgruppen"
                        }
                      ],
                      "label": "Community"
                    },
                    {
                      "children": [
                        {
                          "label": "Engagier dich!",
                          "id": 19869
                        },
                        {
                          "children": [
                            {
                              "label": "als Schüler*in",
                              "id": 21511
                            },
                            {
                              "label": "als Lehrer*in",
                              "id": 21526
                            },
                            {
                              "label": "als Student*in",
                              "id": 21549
                            },
                            {
                              "label": "als Uni / Lehrstuhl",
                              "id": 21551
                            },
                            {
                              "label": "als Softwareentwickler*in",
                              "id": 21555
                            },
                            {
                              "label": "als Organisation",
                              "id": 21559
                            },
                            {
                              "label": "als Unternehmen",
                              "id": 21561
                            },
                            {
                              "label": "in anderen Sprachen",
                              "id": 23320
                            },
                            {
                              "label": "als Bildungsbegeisterte*r",
                              "id": 21557
                            }
                          ],
                          "label": "Was kann ich tun"
                        },
                        {
                          "label": "Spenden",
                          "id": 21565
                        },
                        {
                          "label": "Praktika & Jobs",
                          "id": 21563
                        }
                      ],
                      "label": "Mitmachen"
                    },
                    {
                      "children": [
                        {
                          "label": "Biologie Startseite",
                          "id": 23950
                        },
                        {
                          "label": "Thema auswählen",
                          "id": 23362
                        },
                        {
                          "label": "Lehrplan auswählen",
                          "id": 23362
                        },
                        {
                          "children": [
                            {
                              "label": "Neu hier?",
                              "id": 25017
                            },
                            {
                              "label": "Aktuelles und Planung",
                              "id": 27203
                            },
                            {
                              "label": "Übersicht aller Diskussionen",
                              "url": "/discussions/23382"
                            },
                            {
                              "label": "Richtlinien",
                              "id": 25019
                            },
                            {
                              "label": "Neue Bearbeitungen",
                              "url": "/biologie/entity/trash-bin"
                            },
                            {
                              "label": "Taxonomy bearbeiten",
                              "url": "/taxonomy/term/organize/23362"
                            },
                            {
                              "label": "Papierkorb",
                              "url": "/biologie/entity/trash-bin"
                            }
                          ],
                          "label": "Biologie Community"
                        }
                      ],
                      "label": "Biologie",
                      "id": 23950
                    },
                    {
                      "children": [
                        {
                          "label": "Englisch Startseite",
                          "id": 25985
                        },
                        {
                          "label": "Thema auswählen",
                          "id": 25979
                        },
                        {
                          "children": [
                            {
                              "label": "Neu hier?",
                              "id": 26874
                            },
                            {
                              "label": "Taxonomy bearbeiten",
                              "url": "/taxonomy/term/organize/25979"
                            },
                            {
                              "label": "Neue Bearbeitungen",
                              "url": "/Englisch/entity/trash-bin"
                            },
                            {
                              "label": "Übersicht aller Diskussionen",
                              "url": "/discussions/26876"
                            }
                          ],
                          "label": "Englisch Community"
                        }
                      ],
                      "label": "Englisch",
                      "id": 25979
                    },
                    {
                      "children": [
                        {
                          "label": "BWR Startseite",
                          "id": 26524
                        },
                        {
                          "label": "Thema auswählen",
                          "id": 26523
                        },
                        {
                          "children": [
                            {
                              "label": "Taxonomy bearbeiten",
                              "url": "/taxonomy/term/organize/26523"
                            }
                          ],
                          "label": "BWR Community"
                        }
                      ],
                      "label": "BWR",
                      "id": 26523
                    },
                    {
                      "children": [
                        {
                          "label": "Blog auswählen",
                          "url": "/blog"
                        }
                      ],
                      "label": "Blog",
                      "url": "/blog"
                    }
                  ],
                  "instance": "de"
                }
            "#,
        )
        .unwrap();

        assert_eq!(actual, expected);
    }
}
