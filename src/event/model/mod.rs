use thiserror::Error;

pub use self::event::Event;

mod abstract_event;
mod checkout_revision;
mod create_comment;
mod create_entity;
mod create_entity_link;
mod create_entity_revision;
mod create_taxonomy_link;
mod create_taxonomy_term;
mod create_thread;
mod event;
mod event_type;
mod reject_revision;
mod remove_entity_link;
mod remove_taxonomy_link;
mod set_license;
mod set_taxonomy_parent;
mod set_taxonomy_term;
mod set_thread_state;
mod set_uuid_state;
mod unsupported;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Event cannot be fetched because its type is invalid.")]
    InvalidType,
    #[error("Event cannot be fetched because a required field is missing.")]
    MissingRequiredField,
    #[error("Event does not exist.")]
    NotFound,
}
