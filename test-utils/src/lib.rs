use actix_web::body::to_bytes;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::HttpResponse;
pub use assert_json_diff::assert_json_include;
use convert_case::{Case, Casing};
pub use pretty_assertions::assert_eq;
use rand::rngs::StdRng;
use rand::{distributions::Alphanumeric, Rng};
use rand_seeder::Seeder;
use serde_json::{from_slice, from_value};
pub use serde_json::{json, to_value, Value};
use std::collections::HashMap;
use std::str::FromStr;

use server::create_database_pool;
use server::database::Connection;
use server::message::{Message as ServerMessage, MessageResponder};
use server::uuid::abstract_entity_revision::EntityRevisionType;
use server::uuid::{EntityType, TaxonomyType};

pub struct Message<'a> {
    message_type: &'a str,
    payload: Value,
}

impl<'a> Message<'a> {
    pub fn new(message_type: &'a str, payload: Value) -> Self {
        Self {
            message_type,
            payload,
        }
    }

    pub async fn execute_on(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ) -> MessageResult {
        let message = json!({ "type": self.message_type, "payload": self.payload });
        let message = from_value::<ServerMessage>(message).unwrap();
        let http_response = message.handle(Connection::Transaction(transaction)).await;

        MessageResult::new(http_response).await
    }

    pub async fn execute(&self) -> MessageResult {
        self.execute_on(&mut begin_transaction().await).await
    }
}

pub struct MessageResult {
    pub status: StatusCode,
    body: Bytes,
}

impl MessageResult {
    pub async fn new(response: HttpResponse) -> MessageResult {
        MessageResult {
            status: response.status(),
            body: to_bytes(response.into_body()).await.unwrap(),
        }
    }

    pub fn should_be_ok_with<F>(self, assert_func: F)
    where
        F: Fn(Value),
    {
        assert_eq!(self.status, 200);
        assert_func(self.get_json());
    }

    pub fn should_be_ok_with_body(self, expected_result: Value) {
        self.should_be_response(200, expected_result);
    }

    pub fn should_be_ok(self) {
        assert_eq!(self.status, 200);
    }

    pub fn should_be_not_found(self) {
        self.should_be_response(404, Value::Null);
    }

    pub fn should_be_bad_request(self) {
        assert_eq!(self.status, 400);

        let json_body = self.get_json();
        assert_eq!(json_body["success"], false);
        assert!(json_body["reason"].is_string());
        assert!(!json_body["reason"].as_str().unwrap().is_empty());
    }

    pub fn get_json(self) -> Value {
        from_slice(&self.body).unwrap()
    }

    fn should_be_response(self, expected_status: u16, expected_result: Value) {
        assert_eq!(self.status, expected_status);
        assert_eq!(self.get_json(), expected_result);
    }
}

pub fn assert_has_length(value: &Value, length: usize) {
    assert_eq!(value.as_array().unwrap().len(), length);
}

pub async fn begin_transaction<'a>() -> sqlx::Transaction<'a, sqlx::MySql> {
    create_database_pool().await.unwrap().begin().await.unwrap()
}

pub async fn create_new_test_user<'a, E>(executor: E) -> Result<i32, sqlx::Error>
where
    E: sqlx::Acquire<'a, Database = sqlx::MySql>,
{
    let mut transaction = executor.begin().await?;

    sqlx::query!(
        r#"
                INSERT INTO uuid (trashed, discriminator) VALUES (0, "user")
            "#
    )
    .execute(&mut transaction)
    .await?;

    let new_user_id = sqlx::query!("SELECT LAST_INSERT_ID() as id FROM uuid")
        .fetch_one(&mut transaction)
        .await?
        .id as i32;

    let mut rng: StdRng = Seeder::from(SEED).make_rng();

    sqlx::query!(
        r#"
                INSERT INTO user (id, username, email, password, token)
                VALUES (?, ?, ?, ?, ?)
            "#,
        new_user_id,
        random_string(&mut rng, 10),
        random_string(&mut rng, 10),
        "",
        random_string(&mut rng, 10)
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(new_user_id)
}

pub async fn set_description<'a, E>(
    user_id: i32,
    description: &str,
    executor: E,
) -> Result<(), sqlx::Error>
where
    E: sqlx::mysql::MySqlExecutor<'a>,
{
    sqlx::query!(
        "update user set description = ? where id = ?",
        description,
        user_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn set_email<'a, E>(user_id: i32, email: &str, executor: E) -> Result<(), sqlx::Error>
where
    E: sqlx::mysql::MySqlExecutor<'a>,
{
    sqlx::query!("update user set email = ? where id = ?", email, user_id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn get_email<'a, E>(user_id: i32, executor: E) -> Result<String, sqlx::Error>
where
    E: sqlx::mysql::MySqlExecutor<'a>,
{
    Ok(sqlx::query!("SELECT email FROM user WHERE id = ?", user_id)
        .fetch_one(executor)
        .await?
        .email as String)
}

pub async fn set_entity_revision_field<'a>(
    revision_id: i32,
    field: &str,
    value: &str,
    executor: impl sqlx::Acquire<'a, Database = sqlx::MySql>,
) -> Result<(), sqlx::Error> {
    let mut transaction = executor.begin().await?;

    if sqlx::query!(
        "update entity_revision_field set value = ? where id = ? and field = ?",
        value,
        revision_id,
        value
    )
    .execute(&mut transaction)
    .await?
    .rows_affected()
        == 0
    {
        sqlx::query!(
            "insert into entity_revision_field (entity_revision_id, field, value) values (?, ?, ?)",
            revision_id,
            field,
            value
        )
        .execute(&mut transaction)
        .await?;
    };
    transaction.commit().await?;
    Ok(())
}

pub fn from_value_to_taxonomy_type(value: Value) -> TaxonomyType {
    let type_camel_case = value.as_str().unwrap();
    let type_kebab_case = type_camel_case.to_case(Case::Kebab);
    TaxonomyType::from_str(type_kebab_case.as_str()).unwrap()
}

pub fn from_value_to_entity_type(value: Value) -> EntityType {
    from_value(value).unwrap()
}

pub const ALLOWED_TAXONOMY_TYPES_CREATE: [TaxonomyType; 2] =
    [TaxonomyType::Topic, TaxonomyType::TopicFolder];

pub async fn assert_event_revision_ok(
    revision_id: Value,
    entity_id: i32,
    executor: &mut sqlx::Transaction<'_, sqlx::MySql>,
) {
    Message::new(
        "EventsQuery",
        json!({ "first": 1, "objectId": revision_id }),
    )
    .execute_on(executor)
    .await
    .should_be_ok_with(|result| {
        assert_json_include!(
            actual: &result["events"][0],
            expected: json!({
                "__typename": "CreateEntityRevisionNotificationEvent",
                "objectId": revision_id,
                "entityId": entity_id,
                "entityRevisionId": revision_id

            })
        );
    });
}

pub struct EntityTestWrapper<'a> {
    pub revision_type: EntityRevisionType,
    pub typename: EntityType,
    pub entity_id: i32,
    pub parent_id: Option<i32>,
    pub taxonomy_term_id: Option<i32>,
    pub query_fields: Option<HashMap<&'a str, &'a str>>,
    own_field_keys: Vec<&'a str>,
}

impl EntityTestWrapper<'static> {
    pub fn fields(&self) -> HashMap<&str, &str> {
        let all_entity_fields: HashMap<&str, &str> = HashMap::from([
            ("content", "test content"),
            ("description", "test description"),
            ("metaDescription", "test metaDescription"),
            ("metaTitle", "test metaTitle"),
            ("title", "test title"),
            ("url", "test url"),
            ("cohesive", "true"),
        ]);

        let mut fields = all_entity_fields.clone();
        fields.retain(|key, _| self.own_field_keys.contains(key));
        fields
    }

    pub fn all() -> [Self; 10] {
        [
            EntityTestWrapper {
                revision_type: EntityRevisionType::Applet,
                typename: EntityType::Applet,
                entity_id: 35596,
                parent_id: None,
                own_field_keys: vec!["content", "title", "metaTitle", "metaDescription", "url"],
                query_fields: None,
                taxonomy_term_id: Some(7),
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::Article,
                typename: EntityType::Article,
                entity_id: 1503,
                parent_id: None,
                own_field_keys: vec!["content", "title", "metaTitle", "metaDescription"],
                query_fields: None,
                taxonomy_term_id: Some(7),
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::Course,
                typename: EntityType::Course,
                entity_id: 18275,
                parent_id: None,
                own_field_keys: vec!["description", "title", "metaDescription"],
                query_fields: Some(HashMap::from([
                    ("content", "test description"),
                    ("metaDescription", "test metaDescription"),
                    ("title", "test title"),
                ])),
                taxonomy_term_id: Some(7),
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::CoursePage,
                typename: EntityType::CoursePage,
                entity_id: 18521,
                parent_id: Some(18514),
                own_field_keys: vec!["content", "title"],
                query_fields: None,
                taxonomy_term_id: None,
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::Event,
                typename: EntityType::Event,
                entity_id: 35554,
                parent_id: None,
                own_field_keys: vec!["content", "title", "metaTitle", "metaDescription"],
                query_fields: None,
                taxonomy_term_id: Some(7),
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::Exercise,
                typename: EntityType::Exercise,
                entity_id: 2327,
                parent_id: None,
                own_field_keys: vec!["content"],
                query_fields: None,
                taxonomy_term_id: Some(7),
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::ExerciseGroup,
                typename: EntityType::ExerciseGroup,
                entity_id: 2217,
                parent_id: None,
                own_field_keys: vec!["content", "cohesive"],
                query_fields: Some(HashMap::from([
                    ("content", "test content"),
                    // TODO: missing test due to mismatched type
                    // ("cohesive", true),
                ])),
                taxonomy_term_id: Some(7),
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::GroupedExercise,
                typename: EntityType::GroupedExercise,
                entity_id: 2219,
                parent_id: Some(2217),
                own_field_keys: vec!["content"],
                query_fields: None,
                taxonomy_term_id: None,
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::Solution,
                typename: EntityType::Solution,
                entity_id: 2221,
                parent_id: Some(2219),
                own_field_keys: vec!["content"],
                query_fields: None,
                taxonomy_term_id: None,
            },
            EntityTestWrapper {
                revision_type: EntityRevisionType::Video,
                typename: EntityType::Video,
                entity_id: 16078,
                parent_id: None,
                own_field_keys: vec!["content", "title", "description"],
                query_fields: Some(HashMap::from([
                    ("url", "test content"),
                    ("content", "test description"),
                    ("title", "test title"),
                ])),
                taxonomy_term_id: Some(7),
            },
        ]
    }
}

pub async fn count_taxonomy_entity_links<'a, E>(
    child_id: i32,
    taxonomy_term_id: i32,
    executor: E,
) -> i32
where
    E: sqlx::mysql::MySqlExecutor<'a>,
{
    sqlx::query!(
        r#"
            SELECT COUNT(*) AS count
                FROM term_taxonomy_entity
                WHERE entity_id = ?
                    AND term_taxonomy_id = ?
            "#,
        child_id,
        taxonomy_term_id
    )
    .fetch_one(executor)
    .await
    .unwrap()
    .count as i32
}

const SEED: &str = "YOLOBIRD";

fn random_string(rng: &mut StdRng, length: usize) -> String {
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
