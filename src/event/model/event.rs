use std::convert::{TryFrom, TryInto};

use serde::Serialize;
use sqlx::MySqlPool;

use super::abstract_event::AbstractEvent;
use super::create_comment::CreateComment;
use super::create_entity::CreateEntity;
use super::create_entity_revision::CreateEntityRevision;
use super::create_thread::CreateThread;
use super::entity_link::EntityLink;
use super::event_type::EventType;
use super::revision::Revision;
use super::set_license::SetLicense;
use super::set_taxonomy_parent::SetTaxonomyParent;
use super::set_thread_state::SetThreadState;
use super::set_uuid_state::SetUuidState;
use super::taxonomy_link::TaxonomyLink;
use super::taxonomy_term::TaxonomyTerm;
use super::unsupported::Unsupported;
use super::EventError;
use crate::database::Executor;

#[derive(Serialize)]
pub struct Event {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    #[serde(flatten)]
    pub concrete_event: ConcreteEvent,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ConcreteEvent {
    SetThreadState(SetThreadState),
    CreateComment(CreateComment),
    CreateThread(CreateThread),
    CreateEntity(CreateEntity),
    SetLicense(SetLicense),
    CreateEntityLink(EntityLink),
    RemoveEntityLink(EntityLink),
    CreateEntityRevision(CreateEntityRevision),
    CheckoutRevision(Revision),
    RejectRevision(Revision),
    CreateTaxonomyLink(TaxonomyLink),
    RemoveTaxonomyLink(TaxonomyLink),
    CreateTaxonomyTerm(TaxonomyTerm),
    SetTaxonomyTerm(TaxonomyTerm),
    SetTaxonomyParent(SetTaxonomyParent),
    SetUuidState(SetUuidState),
    Unsupported(Unsupported),
}

impl Event {
    pub async fn fetch(id: i32, pool: &MySqlPool) -> Result<Event, EventError> {
        let abstract_event = AbstractEvent::fetch(id, pool).await?;
        abstract_event.try_into()
    }

    pub async fn fetch_via_transaction<'a, E>(id: i32, executor: E) -> Result<Event, EventError>
    where
        E: Executor<'a>,
    {
        let abstract_event = AbstractEvent::fetch_via_transaction(id, executor).await?;
        abstract_event.try_into()
    }
}

impl TryFrom<AbstractEvent> for Event {
    type Error = EventError;

    fn try_from(abstract_event: AbstractEvent) -> Result<Self, Self::Error> {
        let abstract_event_ref = &abstract_event;
        let concrete_event = match abstract_event_ref.__typename {
            EventType::CheckoutRevision => {
                ConcreteEvent::CheckoutRevision(abstract_event_ref.try_into()?)
            }
            EventType::CreateComment => {
                ConcreteEvent::CreateComment(abstract_event_ref.try_into()?)
            }
            EventType::CreateEntity => ConcreteEvent::CreateEntity(abstract_event_ref.into()),
            EventType::CreateEntityLink => {
                ConcreteEvent::CreateEntityLink(abstract_event_ref.try_into()?)
            }
            EventType::CreateEntityRevision => {
                ConcreteEvent::CreateEntityRevision(abstract_event_ref.try_into()?)
            }
            EventType::CreateTaxonomyLink => {
                ConcreteEvent::CreateTaxonomyLink(abstract_event_ref.try_into()?)
            }
            EventType::CreateTaxonomyTerm => {
                ConcreteEvent::CreateTaxonomyTerm(abstract_event_ref.into())
            }
            EventType::CreateThread => ConcreteEvent::CreateThread(abstract_event_ref.try_into()?),
            EventType::RejectRevision => {
                ConcreteEvent::RejectRevision(abstract_event_ref.try_into()?)
            }
            EventType::RemoveEntityLink => {
                ConcreteEvent::RemoveEntityLink(abstract_event_ref.try_into()?)
            }
            EventType::RemoveTaxonomyLink => {
                ConcreteEvent::RemoveTaxonomyLink(abstract_event_ref.try_into()?)
            }
            EventType::SetLicense => ConcreteEvent::SetLicense(abstract_event_ref.into()),
            EventType::SetTaxonomyParent => {
                ConcreteEvent::SetTaxonomyParent(abstract_event_ref.try_into()?)
            }
            EventType::SetTaxonomyTerm => ConcreteEvent::SetTaxonomyTerm(abstract_event_ref.into()),
            EventType::SetThreadState => ConcreteEvent::SetThreadState(abstract_event_ref.into()),
            EventType::SetUuidState => ConcreteEvent::SetUuidState(abstract_event_ref.into()),
            EventType::Unsupported => ConcreteEvent::Unsupported(abstract_event_ref.into()),
        };

        Ok(Self {
            abstract_event,
            concrete_event,
        })
    }
}
