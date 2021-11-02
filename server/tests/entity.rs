#[cfg(test)]
mod tests {
    use test_utils::*;

    #[actix_rt::test]
    async fn unrevised_entities_query() {
        let r = Message::new("UnrevisedEntitiesQuery", json!({}))
            .execute()
            .await;

        assert_ok(
            r,
            json!({ "unrevisedEntityIds": [26892, 33582, 34741, 34907, 35247, 35556] }),
        )
        .await;
    }
}
