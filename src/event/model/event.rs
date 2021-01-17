use anyhow::Result;
use serde::Serialize;
use serlo_org_database_layer::format_datetime;
use sqlx::MySqlPool;

use crate::event::model::checkout_revision::CheckoutRevision;
use crate::event::model::create_comment::CreateComment;
use crate::event::model::create_entity::CreateEntity;
use crate::event::model::create_entity_link::CreateEntityLink;
use crate::event::model::create_entity_revision::CreateEntityRevision;
use crate::event::model::create_taxonomy_link::CreateTaxonomyLink;
use crate::event::model::create_taxonomy_term::CreateTaxonomyTerm;
use crate::event::model::create_thread::CreateThread;
use crate::event::model::reject_revision::RejectRevision;
use crate::event::model::remove_entity_link::RemoveEntityLink;
use crate::event::model::remove_taxonomy_link::RemoveTaxonomyLink;
use crate::event::model::set_license::SetLicense;
use crate::event::model::set_taxonomy_parent::SetTaxonomyParent;
use crate::event::model::set_taxonomy_term::SetTaxonomyTerm;
use crate::event::model::set_thread_state::SetThreadState;
use crate::event::model::set_uuid_state::SetUuidState;
use crate::event::model::unsupported::Unsupported;

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

pub struct CommonEventData {
    pub id: i32,
    pub name: String,
    pub uuid_id: i32,
    pub actor_id: i32,
    pub date: String,
    pub instance: String,
    pub parameter_uuid_id: Option<i32>,
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

        let e = CommonEventData {
            id: event_fut.id as i32,
            name: event_fut.name,
            uuid_id: event_fut.uuid_id as i32,
            actor_id: event_fut.actor_id as i32,
            date: format_datetime(&event_fut.date),
            instance: event_fut.subdomain,
            parameter_uuid_id: event_fut.parameter_uuid_id.map(|id| id as i32),
        };

        let event = match e.name.as_str() {
            "discussion/comment/archive" => Event::SetThreadState(SetThreadState::build(e, true)),
            "discussion/comment/restore" => Event::SetThreadState(SetThreadState::build(e, false)), //maybe not set threadid?
            "discussion/comment/create" => Event::CreateComment(CreateComment::build(e)),
            "discussion/create" => Event::CreateThread(CreateThread::build(e)),
            "entity/create" => Event::CreateEntity(CreateEntity::build(e)),
            "license/object/set" => Event::SetLicense(SetLicense::build(e)),
            "entity/link/create" => Event::CreateEntityLink(CreateEntityLink::build(e)),
            "entity/link/remove" => Event::RemoveEntityLink(RemoveEntityLink::build(e)),
            "entity/revision/add" => Event::CreateEntityRevision(CreateEntityRevision::build(e)),
            "entity/revision/checkout" => {
                let repository_id = fetch_parameter_uuid_id(e.id, "repository", &pool).await;
                let reason = fetch_parameter_string(e.id, "reason", &pool)
                    .await
                    .unwrap_or("".to_string());
                Event::CheckoutRevision(CheckoutRevision::build(e, repository_id.unwrap(), reason))
            }
            "entity/revision/reject" => {
                let repository_id = fetch_parameter_uuid_id(e.id, "repository", &pool).await;
                let reason = fetch_parameter_string(e.id, "reason", &pool)
                    .await
                    .unwrap_or("".to_string());
                Event::RejectRevision(RejectRevision::build(e, repository_id.unwrap(), reason))
            }
            "taxonomy/term/associate" => Event::CreateTaxonomyLink(CreateTaxonomyLink::build(e)),
            "taxonomy/term/dissociate" => Event::RemoveTaxonomyLink(RemoveTaxonomyLink::build(e)),
            "taxonomy/term/create" => Event::CreateTaxonomyTerm(CreateTaxonomyTerm::build(e)),
            "taxonomy/term/update" => Event::SetTaxonomyTerm(SetTaxonomyTerm::build(e)),
            "taxonomy/term/parent/change" => {
                let from = fetch_parameter_uuid_id(e.id, "from", &pool).await;
                let to = fetch_parameter_uuid_id(e.id, "to", &pool).await;
                Event::SetTaxonomyParent(SetTaxonomyParent::build(e, from, to))
            }
            "uuid/restore" => Event::SetUuidState(SetUuidState::build(e, false)),
            "uuid/trash" => Event::SetUuidState(SetUuidState::build(e, true)),
            _ => Event::Unsupported(Unsupported::build(e)),
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

async fn fetch_parameter_string(id: i32, parameter_name: &str, pool: &MySqlPool) -> Option<String> {
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
}
