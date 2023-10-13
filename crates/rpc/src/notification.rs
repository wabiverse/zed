use crate::proto;
use serde::{Deserialize, Serialize};
use serde_json::{map, Value};
use strum::{EnumVariantNames, VariantNames as _};

const KIND: &'static str = "kind";
const ACTOR_ID: &'static str = "actor_id";

/// A notification that can be stored, associated with a given user.
///
/// This struct is stored in the collab database as JSON, so it shouldn't be
/// changed in a backward-incompatible way. For example, when renaming a
/// variant, add a serde alias for the old name.
///
/// When a notification is initiated by a user, use the `actor_id` field
/// to store the user's id.
#[derive(Debug, Clone, PartialEq, Eq, EnumVariantNames, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Notification {
    ContactRequest {
        actor_id: u64,
    },
    ContactRequestAccepted {
        actor_id: u64,
    },
    ChannelInvitation {
        actor_id: u64,
        channel_id: u64,
    },
    ChannelMessageMention {
        actor_id: u64,
        channel_id: u64,
        message_id: u64,
    },
}

impl Notification {
    pub fn to_proto(&self) -> proto::Notification {
        let mut value = serde_json::to_value(self).unwrap();
        let mut actor_id = None;
        let value = value.as_object_mut().unwrap();
        let Some(Value::String(kind)) = value.remove(KIND) else {
            unreachable!()
        };
        if let map::Entry::Occupied(e) = value.entry(ACTOR_ID) {
            if e.get().is_u64() {
                actor_id = e.remove().as_u64();
            }
        }
        proto::Notification {
            kind,
            actor_id,
            content: serde_json::to_string(&value).unwrap(),
            ..Default::default()
        }
    }

    pub fn from_proto(notification: &proto::Notification) -> Option<Self> {
        let mut value = serde_json::from_str::<Value>(&notification.content).ok()?;
        let object = value.as_object_mut()?;
        object.insert(KIND.into(), notification.kind.to_string().into());
        if let Some(actor_id) = notification.actor_id {
            object.insert(ACTOR_ID.into(), actor_id.into());
        }
        serde_json::from_value(value).ok()
    }

    pub fn all_variant_names() -> &'static [&'static str] {
        Self::VARIANTS
    }
}

#[test]
fn test_notification() {
    // Notifications can be serialized and deserialized.
    for notification in [
        Notification::ContactRequest { actor_id: 1 },
        Notification::ContactRequestAccepted { actor_id: 2 },
        Notification::ChannelInvitation {
            actor_id: 0,
            channel_id: 100,
        },
        Notification::ChannelMessageMention {
            actor_id: 200,
            channel_id: 30,
            message_id: 1,
        },
    ] {
        let message = notification.to_proto();
        let deserialized = Notification::from_proto(&message).unwrap();
        assert_eq!(deserialized, notification);
    }

    // When notifications are serialized, the `kind` and `actor_id` fields are
    // stored separately, and do not appear redundantly in the JSON.
    let notification = Notification::ContactRequest { actor_id: 1 };
    assert_eq!(notification.to_proto().content, "{}");
}
