use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum RawEventType {
    #[serde(rename = "discussion/comment/archive")]
    ArchiveThread,
    #[serde(rename = "discussion/restore")]
    RestoreThread,
    #[serde(rename = "discussion/comment/create")]
    CreateComment,
    #[serde(rename = "discussion/create")]
    CreateThread,
    #[serde(rename = "entity/create")]
    CreateEntity,
    #[serde(rename = "license/object/set")]
    SetLicense,
    #[serde(rename = "entity/link/create")]
    CreateEntityLink,
    #[serde(rename = "entity/link/remove")]
    RemoveEntityLink,
    #[serde(rename = "entity/revision/add")]
    CreateEntityRevision,
    #[serde(rename = "entity/revision/checkout")]
    CheckoutRevision,
    #[serde(rename = "entity/revision/reject")]
    RejectRevision,
    #[serde(rename = "taxonomy/term/associate")]
    CreateTaxonomyLink,
    #[serde(rename = "taxonomy/term/dissociate")]
    RemoveTaxonomyLink,
    #[serde(rename = "taxonomy/term/create")]
    CreateTaxonomyTerm,
    #[serde(rename = "taxonomy/term/update")]
    SetTaxonomyTerm,
    #[serde(rename = "taxonomy/term/parent/change")]
    SetTaxonomyParent,
    #[serde(rename = "uuid/restore")]
    RestoreUuid,
    #[serde(rename = "uuid/trash")]
    TrashUuid,
}

impl std::str::FromStr for RawEventType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}

impl sqlx::Type<MySql> for RawEventType {
    fn type_info() -> MySqlTypeInfo {
        str::type_info()
    }
}
impl<'q> sqlx::Encode<'q, MySql> for RawEventType {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let decoded = serde_json::to_value(self).unwrap();
        let decoded = decoded.as_str().unwrap();
        decoded.encode_by_ref(buf)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum EventType {
    #[serde(rename = "SetThreadStateNotificationEvent")]
    SetThreadState,
    #[serde(rename = "CreateCommentNotificationEvent")]
    CreateComment,
    #[serde(rename = "CreateThreadNotificationEvent")]
    CreateThread,
    #[serde(rename = "CreateEntityNotificationEvent")]
    CreateEntity,
    #[serde(rename = "SetLicenseNotificationEvent")]
    SetLicense,
    #[serde(rename = "CreateEntityLinkNotificationEvent")]
    CreateEntityLink,
    #[serde(rename = "RemoveEntityLinkNotificationEvent")]
    RemoveEntityLink,
    #[serde(rename = "CreateEntityRevisionNotificationEvent")]
    CreateEntityRevision,
    #[serde(rename = "CheckoutRevisionNotificationEvent")]
    CheckoutRevision,
    #[serde(rename = "RejectRevisionNotificationEvent")]
    RejectRevision,
    #[serde(rename = "CreateTaxonomyLinkNotificationEvent")]
    CreateTaxonomyLink,
    #[serde(rename = "RemoveTaxonomyLinkNotificationEvent")]
    RemoveTaxonomyLink,
    #[serde(rename = "CreateTaxonomyTermNotificationEvent")]
    CreateTaxonomyTerm,
    #[serde(rename = "SetTaxonomyTermNotificationEvent")]
    SetTaxonomyTerm,
    #[serde(rename = "SetTaxonomyParentNotificationEvent")]
    SetTaxonomyParent,
    #[serde(rename = "SetUuidStateNotificationEvent")]
    SetUuidState,
}

impl From<RawEventType> for EventType {
    fn from(raw_event_type: RawEventType) -> Self {
        match raw_event_type {
            RawEventType::ArchiveThread => EventType::SetThreadState,
            RawEventType::RestoreThread => EventType::SetThreadState,
            RawEventType::CreateComment => EventType::CreateComment,
            RawEventType::CreateThread => EventType::CreateThread,
            RawEventType::CreateEntity => EventType::CreateEntity,
            RawEventType::SetLicense => EventType::SetLicense,
            RawEventType::CreateEntityLink => EventType::CreateEntityLink,
            RawEventType::RemoveEntityLink => EventType::RemoveEntityLink,
            RawEventType::CreateEntityRevision => EventType::CreateEntityRevision,
            RawEventType::CheckoutRevision => EventType::CheckoutRevision,
            RawEventType::RejectRevision => EventType::RejectRevision,
            RawEventType::CreateTaxonomyLink => EventType::CreateTaxonomyLink,
            RawEventType::RemoveTaxonomyLink => EventType::RemoveTaxonomyLink,
            RawEventType::CreateTaxonomyTerm => EventType::CreateTaxonomyTerm,
            RawEventType::SetTaxonomyTerm => EventType::SetTaxonomyTerm,
            RawEventType::SetTaxonomyParent => EventType::SetTaxonomyParent,
            RawEventType::RestoreUuid => EventType::SetUuidState,
            RawEventType::TrashUuid => EventType::SetUuidState,
        }
    }
}
