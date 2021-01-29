use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum EventType {
    #[serde(rename(
        serialize = "SetThreadStateNotificationEvent",
        deserialize = "discussion/comment/archive",
        deserialize = "discussion/restore"
    ))]
    SetThreadState,
    #[serde(rename(
        serialize = "CreateCommentNotificationEvent",
        deserialize = "discussion/comment/create"
    ))]
    CreateComment,
    #[serde(rename(
        serialize = "CreateThreadNotificationEvent",
        deserialize = "discussion/create"
    ))]
    CreateThread,
    #[serde(rename(
        serialize = "CreateEntityNotificationEvent",
        deserialize = "entity/create"
    ))]
    CreateEntity,
    #[serde(rename(
        serialize = "SetLicenseNotificationEvent",
        deserialize = "license/object/set"
    ))]
    SetLicense,
    #[serde(rename(
        serialize = "CreateEntityLinkNotificationEvent",
        deserialize = "entity/link/create"
    ))]
    CreateEntityLink,
    #[serde(rename(
        serialize = "RemoveEntityLinkNotificationEvent",
        deserialize = "entity/link/remove"
    ))]
    RemoveEntityLink,
    #[serde(rename(
        serialize = "CreateEntityRevisionNotificationEvent",
        deserialize = "entity/revision/add"
    ))]
    CreateEntityRevision,
    #[serde(rename(
        serialize = "CheckoutRevisionNotificationEvent",
        deserialize = "entity/revision/checkout"
    ))]
    CheckoutRevision,
    #[serde(rename(
        serialize = "RejectRevisionNotificationEvent",
        deserialize = "entity/revision/reject"
    ))]
    RejectRevision,
    #[serde(rename(
        serialize = "CreateTaxonomyLinkNotificationEvent",
        deserialize = "taxonomy/term/associate"
    ))]
    CreateTaxonomyLink,
    #[serde(rename(
        serialize = "RemoveTaxonomyLinkNotificationEvent",
        deserialize = "taxonomy/term/dissociate"
    ))]
    RemoveTaxonomyLink,
    #[serde(rename(
        serialize = "CreateTaxonomyTermNotificationEvent",
        deserialize = "taxonomy/term/create"
    ))]
    CreateTaxonomyTerm,
    #[serde(rename(
        serialize = "SetTaxonomyTermNotificationEvent",
        deserialize = "taxonomy/term/update"
    ))]
    SetTaxonomyTerm,
    #[serde(rename(
        serialize = "SetTaxonomyParentNotificationEvent",
        deserialize = "taxonomy/term/parent/change"
    ))]
    SetTaxonomyParent,
    #[serde(rename(
        serialize = "SetUuidStateNotificationEvent",
        deserialize = "uuid/restore",
        deserialize = "uuid/trash"
    ))]
    SetUuidState,
}

impl std::str::FromStr for EventType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}
