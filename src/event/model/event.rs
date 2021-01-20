use anyhow::Result;
use serde::Serialize;
use serlo_org_database_layer::format_datetime;
use sqlx::MySqlPool;

use super::checkout_revision::CheckoutRevision;
use super::create_comment::CreateComment;
use super::create_entity::CreateEntity;
use super::create_entity_link::CreateEntityLink;
use super::create_entity_revision::CreateEntityRevision;
use super::create_taxonomy_link::CreateTaxonomyLink;
use super::create_taxonomy_term::CreateTaxonomyTerm;
use super::create_thread::CreateThread;
use super::reject_revision::RejectRevision;
use super::remove_entity_link::RemoveEntityLink;
use super::remove_taxonomy_link::RemoveTaxonomyLink;
use super::set_license::SetLicense;
use super::set_taxonomy_parent::SetTaxonomyParent;
use super::set_taxonomy_term::SetTaxonomyTerm;
use super::set_thread_state::SetThreadState;
use super::set_uuid_state::SetUuidState;
use super::unsupported::Unsupported;

// To string

#[derive(Serialize)]
#[serde(untagged)]
pub enum Event {
    SetThreadState(SetThreadState),
    CreateComment(CreateComment),
    CreateThread(CreateThread),
    CreateEntity(CreateEntity),
    SetLicense(SetLicense),
    CreateEntityLink(CreateEntityLink),
    RemoveEntityLink(RemoveEntityLink),
    CreateEntityRevision(CreateEntityRevision),
    CheckoutRevision(CheckoutRevision),
    RejectRevision(RejectRevision),
    CreateTaxonomyLink(CreateTaxonomyLink),
    RemoveTaxonomyLink(RemoveTaxonomyLink),
    CreateTaxonomyTerm(CreateTaxonomyTerm),
    SetTaxonomyTerm(SetTaxonomyTerm),
    SetTaxonomyParent(SetTaxonomyParent),
    SetUuidState(SetUuidState),
    Unsupported(Unsupported),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractEvent {
    pub id: i32,
    pub instance: String,
    pub date: String,
    pub actor_id: i32,
    pub object_id: i32,
    #[serde(skip)]
    pub parameter_uuid_id: i32,
}

impl Event {
    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Event> {
        let event_fut = sqlx::query!(
            r#"
                SELECT e.id, e.actor_id, e.uuid_id, e.date, i.subdomain, ev.name, id.uuid_id AS parameter_uuid_id
                    FROM event_log e
                    LEFT JOIN event_parameter p ON e.id = p.log_id
                    LEFT JOIN event_parameter_uuid id ON p.id = id.event_parameter_id
                    JOIN instance i ON e.instance_id = i.id
                    JOIN event ev ON e.event_id = ev.id
                    WHERE e.id = ?
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        let parameter_uuid_id = event_fut.parameter_uuid_id.map(|id| id as i32);
        let name = event_fut.name;
        let uuid_id = event_fut.uuid_id as i32;

        let abstract_event = AbstractEvent {
            id: event_fut.id as i32,
            instance: event_fut.subdomain.to_string(),
            date: format_datetime(&event_fut.date),
            actor_id: event_fut.actor_id as i32,
            object_id: uuid_id,
            parameter_uuid_id: parameter_uuid_id.unwrap_or(uuid_id),
        };

        let event = match name.as_str() {
            "discussion/comment/archive" => {
                Event::SetThreadState(SetThreadState::new(abstract_event, true))
            }
            "discussion/comment/restore" => {
                Event::SetThreadState(SetThreadState::new(abstract_event, false))
            }
            "discussion/comment/create" => Event::CreateComment(CreateComment::new(abstract_event)),
            "discussion/create" => Event::CreateThread(CreateThread::new(abstract_event)),
            "entity/create" => Event::CreateEntity(CreateEntity::new(abstract_event)),
            "license/object/set" => Event::SetLicense(SetLicense::new(abstract_event)),
            "entity/link/create" => Event::CreateEntityLink(CreateEntityLink::new(abstract_event)),
            "entity/link/remove" => Event::RemoveEntityLink(RemoveEntityLink::new(abstract_event)),
            "entity/revision/add" => {
                Event::CreateEntityRevision(CreateEntityRevision::new(abstract_event))
            }
            "entity/revision/checkout" => {
                let repository_id = fetch_parameter_uuid_id(abstract_event.id, "repository", &pool)
                    .await
                    .unwrap();
                let reason = fetch_parameter_string(abstract_event.id, "reason", &pool).await;
                Event::CheckoutRevision(CheckoutRevision::new(
                    abstract_event,
                    repository_id,
                    reason,
                ))
            }
            "entity/revision/reject" => {
                let repository_id = fetch_parameter_uuid_id(abstract_event.id, "repository", &pool)
                    .await
                    .unwrap();
                let reason = fetch_parameter_string(abstract_event.id, "reason", &pool).await;
                Event::RejectRevision(RejectRevision::new(abstract_event, repository_id, reason))
            }
            "taxonomy/term/associate" => {
                Event::CreateTaxonomyLink(CreateTaxonomyLink::new(abstract_event))
            }
            "taxonomy/term/dissociate" => {
                Event::RemoveTaxonomyLink(RemoveTaxonomyLink::new(abstract_event))
            }
            "taxonomy/term/create" => {
                Event::CreateTaxonomyTerm(CreateTaxonomyTerm::new(abstract_event))
            }
            "taxonomy/term/update" => Event::SetTaxonomyTerm(SetTaxonomyTerm::new(abstract_event)),
            "taxonomy/term/parent/change" => {
                let from = fetch_parameter_uuid_id(abstract_event.id, "from", &pool).await;
                let to = fetch_parameter_uuid_id(abstract_event.id, "to", &pool).await;
                Event::SetTaxonomyParent(SetTaxonomyParent::new(abstract_event, from, to))
            }
            "uuid/restore" => Event::SetUuidState(SetUuidState::new(abstract_event, false)),
            "uuid/trash" => Event::SetUuidState(SetUuidState::new(abstract_event, true)),
            _ => Event::Unsupported(Unsupported::new(abstract_event, name)),
        };

        Ok(event)
    }
}

async fn fetch_parameter_uuid_id(id: i32, parameter_name: &str, pool: &MySqlPool) -> Option<i32> {
    sqlx::query!(
        r#"
            SELECT id.uuid_id FROM event_parameter p
                JOIN event_parameter_name n ON n.name = ?
                JOIN event_parameter_uuid id ON id.event_parameter_id = p.id
                WHERE p.name_id = n.id AND p.log_id = ?
        "#,
        parameter_name,
        id,
    )
    .fetch_one(pool)
    .await
    .ok()
    .map(|o| o.uuid_id as i32)
}

async fn fetch_parameter_string(id: i32, parameter_name: &str, pool: &MySqlPool) -> String {
    sqlx::query!(
        r#"
            SELECT s.value FROM event_parameter p
                JOIN event_parameter_name n ON n.name = ?
                JOIN event_parameter_string s ON s.event_parameter_id = p.id
                WHERE p.name_id = n.id AND p.log_id = ?
        "#,
        parameter_name,
        id,
    )
    .fetch_one(pool)
    .await
    .ok()
    .map(|o| o.value)
    .unwrap_or_else(|| "".to_string())
}
