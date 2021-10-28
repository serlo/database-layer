#[cfg(test)]
mod tests {
    use test_utils::*;

    #[actix_rt::test]
    async fn unrevised_entities_query() {
        let response = Message::new("UnrevisedEntitiesQuery", json!({}))
            .execute()
            .await;

        assert_ok(
            response,
            json!({ "unrevisedEntityIds": [26892, 33582, 34741, 34907, 35247, 35556] }),
        )
        .await;
    }
}
