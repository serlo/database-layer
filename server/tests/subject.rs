#[cfg(test)]
mod subjects_query {
    use test_utils::*;

    #[actix_rt::test]
    async fn returns_list_of_subjects() {
        Message::new("SubjectsQuery", Value::Null)
            .execute()
            .await
            .should_be_ok_with_body(json!({
              "subjects": [
                {
                  "instance": "de",
                  "taxonomyTermId": 5
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 17744
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 18230
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 23362
                },
                {
                  "instance": "en",
                  "taxonomyTermId": 23593
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 25712
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 25979
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 26523
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 33894
                },
                {
                  "instance": "de",
                  "taxonomyTermId": 35608
                }
              ]
            }));
    }
}
