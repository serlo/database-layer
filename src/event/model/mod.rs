use thiserror::Error;

pub use self::abstract_event::*;
pub use self::create_comment::*;
pub use self::create_entity::*;
pub use self::create_entity_revision::*;
pub use self::create_thread::*;
pub use self::entity_link::*;
pub use self::event::*;
pub use self::event_type::*;
pub use self::revision::*;
pub use self::set_license::*;
pub use self::set_taxonomy_parent::*;
pub use self::set_thread_state::*;
pub use self::set_uuid_state::*;
pub use self::taxonomy_link::*;
pub use self::taxonomy_term::*;

mod abstract_event;
mod create_comment;
mod create_entity;
mod create_entity_revision;
mod create_thread;
mod entity_link;
mod event;
mod event_type;
mod revision;
mod set_license;
mod set_taxonomy_parent;
mod set_thread_state;
mod set_uuid_state;
mod taxonomy_link;
mod taxonomy_term;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event cannot be fetched because of a database error: {inner:?}.")]
    DatabaseError { inner: sqlx::Error },
    #[error("Event cannot be fetched because its type is invalid.")]
    InvalidType,
    #[error("Event cannot be fetched because its instance is invalid.")]
    InvalidInstance,
    #[error("Event cannot be fetched because a required field is missing.")]
    MissingRequiredField,
    #[error("Event does not exist.")]
    NotFound,
}

impl From<sqlx::Error> for EventError {
    fn from(inner: sqlx::Error) -> Self {
        EventError::DatabaseError { inner }
    }
}
