use anyhow::Result;
use futures::join;
use serde::Serialize;
use serlo_org_database_layer::format_datetime;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "__typename"))]
    pub __typename: Option<String>,
    pub id: i32,
    pub instance: String,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<i32>,
    pub object_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trashed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_revision_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxonomy_term_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i32>,

    // error return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Event {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Option<Event>> {
        let event_fut = sqlx::query!(
            r#"
                SELECT e.event_id, e.actor_id, e.uuid_id, e.instance_id, e.date, i.subdomain, event.name,
                    GROUP_CONCAT(p.id) as parameter_ids
                    FROM event_log e
                    LEFT JOIN event_parameter p ON e.id = p.log_id
                    LEFT JOIN instance i ON e.instance_id = i.id
                    JOIN event ON e.event_id = event.id
                    WHERE e.id = ?
                    GROUP BY e.event_id, e.actor_id, e.uuid_id, e.instance_id, e.date
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        let uuid_id = event_fut.uuid_id as i32;
        let name = event_fut.name;
        let parameter_ids = event_fut.parameter_ids;

        // query parameters
        let object_uuid_id = query_object_uuid_id(&name, &parameter_ids, &pool).await;
        let repository_uuid_id = query_repository_uuid_id(&name, &parameter_ids, &pool).await;
        let parent_uuid_id = query_parent_uuid_id(&name, &parameter_ids, &pool).await;
        let on_uuid_id = query_on_uuid_id(&name, &parameter_ids, &pool).await;
        let thread_uuid_id = query_thread_uuid_id(&name, &parameter_ids, &pool).await;
        let reason = query_reason_string(&name, &parameter_ids, &pool).await;
        let from_and_to = query_from_and_to_ids(&name, &parameter_ids, &pool).await;

        // TODO: Should we keep that consistency with legacy?
        if name == "discussion/restore" {
            return Ok(Some(Event {
                __typename: None,
                id,
                date: format_datetime(&event_fut.date),
                instance: event_fut.subdomain.unwrap(),
                object_id: get_object_id(&name, uuid_id, on_uuid_id),
                r#type: Some(String::from("discussion/restore")),
                error: Some(String::from("unsupported")),
                archived: None,
                actor_id: None,
                thread_id: None,
                comment_id: None,
                child_id: None,
                entity_revision_id: None,
                revision_id: None,
                trashed: None,
                taxonomy_term_id: None,
                entity_id: None,
                repository_id: None,
                reason: None,
                parent_id: None,
                previous_parent_id: None,
            }));
        }

        let typename = get_typename(&name);
        let parent_id = get_parent_id(&name, uuid_id, parent_uuid_id, &from_and_to);
        let previous_parent_id = get_previous_parent_id(&name, &from_and_to);

        if typename.as_str() == "SetTaxonomyParentNotificationEvent"
            && parent_id.unwrap().is_none()
            && previous_parent_id.is_none()
        {
            return Ok(None);
        }

        Ok(Some(Event {
            // for all
            __typename: Some(typename),
            id,
            instance: event_fut.subdomain.unwrap(),
            date: format_datetime(&event_fut.date),
            actor_id: Some(event_fut.actor_id as i32),
            object_id: get_object_id(&name, uuid_id, on_uuid_id),

            // for some
            archived: get_archived(&name),
            thread_id: get_thread_id(&name, uuid_id, thread_uuid_id),
            comment_id: get_comment_id(&name, uuid_id),
            child_id: get_child_id(&name, uuid_id, object_uuid_id),
            entity_revision_id: get_entity_revision_id(&name, uuid_id),
            revision_id: get_revision_id(&name, uuid_id),
            trashed: get_trashed(&name),
            taxonomy_term_id: get_taxonomy_term_id(&name, uuid_id),
            entity_id: get_entity_id(&name, uuid_id, repository_uuid_id),
            repository_id: get_repository_id(&name, uuid_id, repository_uuid_id),
            reason: get_reason(&name, reason),
            parent_id,
            previous_parent_id,

            // never
            r#type: None,
            error: None,
        }))
    }
}

fn get_typename(name: &str) -> String {
    let typename = match name {
        "discussion/comment/archive" => "SetThreadStateNotificationEvent",
        "discussion/comment/restore" => "SetThreadStateNotificationEvent",
        "discussion/comment/create" => "CreateCommentNotificationEvent",
        "discussion/create" => "CreateThreadNotificationEvent",
        "entity/create" => "CreateEntityNotificationEvent",
        "license/object/set" => "SetLicenseNotificationEvent",
        "entity/link/create" => "CreateEntityLinkNotificationEvent",
        "entity/link/remove" => "RemoveEntityLinkNotificationEvent",
        "entity/revision/add" => "CreateEntityRevisionNotificationEvent",
        "entity/revision/checkout" => "CheckoutRevisionNotificationEvent",
        "entity/revision/reject" => "RejectRevisionNotificationEvent",
        "taxonomy/term/associate" => "CreateTaxonomyLinkNotificationEvent",
        "taxonomy/term/dissociate" => "RemoveTaxonomyLinkNotificationEvent",
        "taxonomy/term/create" => "CreateTaxonomyTermNotificationEvent",
        "taxonomy/term/update" => "SetTaxonomyTermNotificationEvent",
        "taxonomy/term/parent/change" => "SetTaxonomyParentNotificationEvent",
        "uuid/restore" => "SetUuidStateNotificationEvent",
        "uuid/trash" => "SetUuidStateNotificationEvent",
        _ => "",
    };
    String::from(typename)
}

fn get_archived(name: &str) -> Option<bool> {
    match name {
        "discussion/comment/archive" => Some(true),
        "discussion/comment/restore" => Some(false),
        _ => None,
    }
}

fn get_thread_id(name: &str, uuid_id: i32, thread_uuid_id: Option<i32>) -> Option<i32> {
    if name == "discussion/create" {
        return Some(uuid_id);
    }
    match name == "discussion/comment/archive" || name == "discussion/comment/create" {
        true => match thread_uuid_id.is_some() {
            true => thread_uuid_id,
            false => Some(uuid_id),
        },
        false => None,
    }
}

fn get_comment_id(name: &str, uuid_id: i32) -> Option<i32> {
    match name == "discussion/comment/create" {
        true => Some(uuid_id),
        false => None,
    }
}

fn get_object_id(name: &str, uuid_id: i32, on_uuid_id: Option<i32>) -> i32 {
    if name == "discussion/create" {
        on_uuid_id.unwrap_or(uuid_id)
    } else {
        uuid_id
    }
}

fn get_child_id(name: &str, uuid_id: i32, object_uuid_id: Option<i32>) -> Option<i32> {
    if name == "taxonomy/term/associate" || name == "taxonomy/term/dissociate" {
        return object_uuid_id;
    }

    if name == "entity/link/create"
        || name == "entity/link/remove"
        || name == "taxonomy/term/parent/change"
    {
        return Some(uuid_id);
    }

    None
}

fn get_entity_revision_id(name: &str, uuid_id: i32) -> Option<i32> {
    match name {
        "entity/revision/add" => Some(uuid_id),
        _ => None,
    }
}

fn get_revision_id(name: &str, uuid_id: i32) -> Option<i32> {
    match name == "entity/revision/checkout" || name == "entity/revision/reject" {
        true => Some(uuid_id),
        false => None,
    }
}

fn get_trashed(name: &str) -> Option<bool> {
    match name {
        "uuid/restore" => Some(false),
        "uuid/trash" => Some(true),
        _ => None,
    }
}

fn get_taxonomy_term_id(name: &str, uuid_id: i32) -> Option<i32> {
    match name == "taxonomy/term/create" || name == "taxonomy/term/update" {
        true => Some(uuid_id),
        false => None,
    }
}

fn get_entity_id(name: &str, uuid_id: i32, repository_uuid_id: Option<i32>) -> Option<i32> {
    match name {
        "entity/create" => Some(uuid_id),
        "entity/revision/add" => repository_uuid_id,
        _ => None,
    }
}

fn get_reason(name: &str, reason: Option<String>) -> Option<String> {
    match name == "entity/revision/checkout" || name == "entity/revision/reject" {
        true => match reason.is_some() {
            true => reason,
            false => Some(String::from("")),
        },
        false => None,
    }
}

fn get_repository_id(name: &str, uuid_id: i32, repository_uuid_id: Option<i32>) -> Option<i32> {
    if name == "license/object/set" {
        return Some(uuid_id);
    };

    match name == "entity/revision/checkout" || name == "entity/revision/reject" {
        true => repository_uuid_id,
        false => None,
    }
}

fn get_parent_id(
    name: &str,
    uuid_id: i32,
    parent_uuid_id: Option<i32>,
    from_and_to: &(Option<i32>, Option<i32>),
) -> Option<Option<i32>> {
    // Wrapping option decides if it should be serialized
    match name {
        "entity/link/create" | "entity/link/remove" => Some(parent_uuid_id),
        "taxonomy/term/associate" | "taxonomy/term/dissociate" => Some(Some(uuid_id)),
        "taxonomy/term/parent/change" => match from_and_to.1 {
            Some(to) => Some(Some(to)),
            None => Some(None),
        },
        _ => None,
    }
}

fn get_previous_parent_id(name: &str, from_and_to: &(Option<i32>, Option<i32>)) -> Option<i32> {
    if name == "taxonomy/term/parent/change" {
        return from_and_to.0;
    }
    None
}

async fn query_repository_uuid_id(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<i32> {
    match name == "entity/revision/add"
        || name == "entity/revision/checkout"
        || name == "entity/revision/reject"
    {
        true => query_parameter_uuid_id(parameter_ids, pool).await,
        false => None,
    }
}

async fn query_on_uuid_id(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<i32> {
    match name == "discussion/create" {
        true => query_parameter_uuid_id(parameter_ids, pool).await,
        false => None,
    }
}

async fn query_object_uuid_id(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<i32> {
    match name == "taxonomy/term/associate" || name == "taxonomy/term/dissociate" {
        true => query_parameter_uuid_id(parameter_ids, pool).await,
        false => None,
    }
}

async fn query_parent_uuid_id(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<i32> {
    match name == "entity/link/create" || name == "entity/link/remove" {
        true => query_parameter_uuid_id(parameter_ids, pool).await,
        false => None,
    }
}

async fn query_thread_uuid_id(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<i32> {
    match name == "discussion/comment/archive" || name == "discussion/comment/create" {
        true => query_parameter_uuid_id(parameter_ids, pool).await,
        false => None,
    }
}

async fn query_parameter_uuid_id(parameter_ids: &Option<String>, pool: &MySqlPool) -> Option<i32> {
    if parameter_ids.is_none() {
        return None;
    }
    let uuid_id_fut = sqlx::query!(
        "
            SELECT uuid_id FROM event_parameter_uuid
            WHERE FIND_IN_SET(event_parameter_id, ?)
        ",
        parameter_ids
    )
    .fetch_one(pool)
    .await;

    uuid_id_fut.ok().map(|value| value.uuid_id as i32)
}

async fn query_reason_string(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<String> {
    match name == "entity/revision/checkout" || name == "entity/revision/reject" {
        true => query_parameter_string(parameter_ids, pool).await,
        false => None,
    }
}

async fn query_parameter_string(
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<String> {
    if parameter_ids.is_none() {
        return None;
    }
    let string_fut = sqlx::query!(
        "
            SELECT value FROM event_parameter_string
            WHERE FIND_IN_SET(event_parameter_id, ?)
        ",
        parameter_ids
    )
    .fetch_one(pool)
    .await;

    string_fut.ok().map(|value| value.value as String)
}

async fn query_from_and_to_ids(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> (Option<i32>, Option<i32>) {
    if name != "taxonomy/term/parent/change" || parameter_ids.is_none() {
        return (None, None);
    }
    // could probably rewritten to return in one query, but it was a lot easier this way
    // also: relies on hardcoded parameter name id (7 = from; 8 = to).
    // okay, or should we query the names event_parameter_name also to check?

    // puh: this one is surprisingly annoying :)
    // legacy queries event_parameter_string also to check for "no parent" string
    // but I think it's okay to set None when there is no uuid_id present

    let from_fut = sqlx::query!(
        r#"
            SELECT u.uuid_id
                FROM event_parameter e
                JOIN event_parameter_uuid u ON e.id = u.event_parameter_id
                WHERE FIND_IN_SET(e.id, ? ) AND e.name_id = 7
                ORDER BY e.name_id
        "#,
        parameter_ids
    )
    .fetch_one(pool);
    let to_fut = sqlx::query!(
        r#"
            SELECT u.uuid_id
                FROM event_parameter e
                JOIN event_parameter_uuid u ON e.id = u.event_parameter_id
                WHERE FIND_IN_SET(e.id, ? ) AND e.name_id = 8
                ORDER BY e.name_id
        "#,
        parameter_ids
    )
    .fetch_one(pool);

    let (from, to) = join!(from_fut, to_fut);
    (
        from.ok().map(|value| value.uuid_id as i32),
        to.ok().map(|value| value.uuid_id as i32),
    )
}
