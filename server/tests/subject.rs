#[cfg(test)]
mod tests {
    use test_utils::*;

    #[actix_rt::test]
    async fn subjects_query() {
        let response = Message::new("SubjectsQuery", Value::Null).execute().await;

        assert_ok(
            response,
            json!({
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
            }),
        )
        .await;
    }
}
