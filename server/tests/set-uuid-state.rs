#[cfg(test)]
mod tests {
    use test_utils::*;

    #[actix_rt::test]
    async fn set_uuid_state_for_untrashable_uuids_fails() {
        for discriminator in ["entityRevision", "user"].iter() {
            let mut transaction = begin_transaction().await;

            let revision_id = sqlx::query!(
                "select id from uuid where discriminator = ? and trashed = false",
                discriminator
            )
            .fetch_one(&mut transaction)
            .await
            .unwrap()
            .id as i32;

            let response = Message::new(
                "UuidSetStateMutation",
                json!({ "ids": [revision_id], "userId": 1, "trashed": true }),
            )
            .execute()
            .await;

            assert_bad_request(
                response,
                format!(
                    "uuid {} with type \"{}\" cannot be deleted via a setState mutation",
                    revision_id, discriminator
                )
                .as_str(),
            )
            .await;
        }
    }
}
