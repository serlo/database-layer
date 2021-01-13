use anyhow::Result;
use database_layer_actix::format_datetime;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub actor_id: i32,
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
    pub object_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
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
}

impl Event {
    pub async fn get_event(id: i32, pool: &MySqlPool) -> Result<Event> {
        let event_fut = sqlx::query!(
            "
            SELECT el.event_id, el.actor_id, el.uuid_id, el.instance_id, el.date, instance.subdomain, event.name,
            GROUP_CONCAT(event_parameter.id) as parameter_ids
            FROM event_log el
            LEFT JOIN instance ON el.instance_id = instance.id
            INNER JOIN event ON el.event_id = event.id
            LEFT JOIN event_parameter ON el.id = event_parameter.log_id
            WHERE el.id = ?
            GROUP BY el.event_id, el.actor_id, el.uuid_id, el.instance_id, el.date
            ",
            id
        )
        .fetch_one(pool)
        .await?;

        let uuid_id = event_fut.uuid_id as i32;
        let name = event_fut.name;
        let paramater_ids = event_fut.parameter_ids;

        //query parameters
        let repository_uuid_id = query_repository_uuid_id(&name, &paramater_ids, &pool).await;
        let object_uuid_id = query_object_uuid_id(&name, &paramater_ids, &pool).await;
        let reason = query_reason_string(&name, &paramater_ids, &pool).await;

        Ok(Event {
            //for all
            __typename: get_typename(&name),
            id: id,
            instance: event_fut.subdomain.unwrap(),
            date: format_datetime(&event_fut.date),
            actor_id: event_fut.actor_id as i32,

            //for some
            archived: get_archived(&name),
            thread_id: get_thread_id(&name, uuid_id),
            comment_id: get_comment_id(&name, uuid_id),
            object_id: get_object_id(&name, uuid_id),
            child_id: get_child_id(&name, uuid_id, object_uuid_id),
            entity_revision_id: get_entity_revision_id(&name, uuid_id),
            revision_id: get_revision_id(&name, uuid_id),
            trashed: get_trashed(&name),
            taxonomy_term_id: get_taxonomy_term_id(&name, uuid_id),
            entity_id: get_entity_id(&name, uuid_id, repository_uuid_id),
            repository_id: get_repository_id(&name, uuid_id, repository_uuid_id),
            reason: get_reason(&name, reason),
            parent_id: get_parent_id(&name, uuid_id, &paramater_ids),
            previous_parent_id: get_previous_parent_id(&name, &paramater_ids),
        })
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

fn get_thread_id(name: &str, uuid_id: i32) -> Option<i32> {
    match name.starts_with("discussion") {
        true => Some(uuid_id),
        false => None,
    }
}

fn get_comment_id(name: &str, uuid_id: i32) -> Option<i32> {
    match name == "discussion/comment/create" {
        true => Some(uuid_id),
        false => None,
    }
}

fn get_object_id(name: &str, uuid_id: i32) -> Option<i32> {
    match name == "discussion/create" {
        true => Some(uuid_id),
        false => None,
    }
}

fn get_child_id(name: &str, uuid_id: i32, object_uuid_id: Option<i32>) -> Option<i32> {
    if name == "taxonomy/term/associate" || name == "taxonomy/term/dissociate" {
        return match object_uuid_id {
            Some(id) => Some(id),
            None => None,
        };
    }

    match name == "entity/link/create"
        || name == "entity/link/remove"
        || name == "taxonomy/term/parent/change"
    {
        true => Some(uuid_id),
        false => None,
    }
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
        "entity/revision/add" => match repository_uuid_id {
            Some(id) => Some(id),
            None => None,
        },
        _ => None,
    }
}

fn get_reason(name: &str, reason: Option<String>) -> Option<String> {
    match name == "entity/revision/checkout" || name == "entity/revision/reject" {
        true => match reason {
            Some(string) => Some(string),
            None => None,
        },
        false => None,
    }
}

fn get_repository_id(name: &str, uuid_id: i32, repository_uuid_id: Option<i32>) -> Option<i32> {
    if name == "license/object/set" {
        return Some(uuid_id);
    };

    match name == "entity/revision/checkout" || name == "entity/revision/reject" {
        true => match repository_uuid_id {
            Some(id) => Some(id),
            None => None,
        },
        false => None,
    }
}

// TODO!
fn get_parent_id(name: &str, uuid_id: i32, parameter_ids: &Option<String>) -> Option<i32> {
    match name {
        "entity/link/create" | "entity/link/remove" => Some(0), //TODO: get parameter parent id $event->getParameter('parent')->getId(),
        "taxonomy/term/associate" | "taxonomy/term/dissociate" => Some(uuid_id),
        "taxonomy/term/parent/change" => {
            Some(0) //TODO: $to != 'no parent' ? $to->getId() : null,
        }
        _ => None,
    }
}

fn get_previous_parent_id(name: &str, parameter_ids: &Option<String>) -> Option<i32> {
    match name == "taxonomy/term/parent/change" {
        true => {
            Some(0) //TODO: $from != 'no parent' ? $from->getId() : null
        }
        false => None,
    }
}

async fn query_from_and_to_ids(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> [Option<i32>; 2] {
    match name == "taxonomy/term/parent/change" {
        true => {
            //TODO: find out how this is supposed to work…
            return [
                query_parameter_uuid_id(parameter_ids, pool).await,
                query_parameter_uuid_id(parameter_ids, pool).await,
            ];
        }
        false => return [None, None],
    }
}

async fn query_repository_uuid_id(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<i32> {
    match name == "entity/revision/add"
        || name == "entity/revision/checkout"
        || name == "entity/revision/add"
        || name == "entity/revision/reject"
    {
        true => return query_parameter_uuid_id(parameter_ids, pool).await,
        false => return None,
    }
}

async fn query_object_uuid_id(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<i32> {
    match name == "taxonomy/term/associate" || name == "taxonomy/term/dissociate" {
        true => return query_parameter_uuid_id(parameter_ids, pool).await,
        false => return None,
    }
}

async fn query_parameter_uuid_id(parameter_ids: &Option<String>, pool: &MySqlPool) -> Option<i32> {
    if parameter_ids.is_none() {
        return None;
    }
    let uuid_id_fut = sqlx::query!(
        "
            SELECT uuid_id FROM event_parameter_uuid
            WHERE event_parameter_id in ( ? )
        ",
        parameter_ids
    )
    .fetch_one(pool)
    .await;

    match uuid_id_fut {
        Ok(value) => return Some(value.uuid_id as i32),
        _ => return None,
    }
}

async fn query_reason_string(
    name: &str,
    parameter_ids: &Option<String>,
    pool: &MySqlPool,
) -> Option<String> {
    match name == "entity/revision/checkout" || name == "entity/revision/reject" {
        true => return query_parameter_string(parameter_ids, pool).await,
        false => return None,
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
            WHERE event_parameter_id in ( ? )
        ",
        parameter_ids
    )
    .fetch_one(pool)
    .await;

    match string_fut {
        Ok(string) => return Some(string.value as String),
        _ => return None,
    }
}

// case 'taxonomy/term/parent/change':
// $from = $event->getParameter('from');
// $to = $event->getParameter('to');

//     'previousParentId' =>
//         $from != 'no parent' ? $from->getId() : null,
//     'parentId' => $to != 'no parent' ? $to->getId() : null,
// ];

// default: 'Unsupported event type',
