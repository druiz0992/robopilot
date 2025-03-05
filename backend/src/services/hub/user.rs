use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::controller::HubReceiver;
use crate::models::hub::{HubChannelName, HubMessage};

/// `HubSubscriptionInfo` holds information about a subscription in the hub, including
/// channel name and associated channel receiver.
type HubSubscriptionInfo = HashMap<HubChannelName, broadcast::Receiver<HubMessage>>;

/// `HubUsers` manages a collection of users in the hub, identified by their UUIDs.
/// Each UUID identifies a collection of channels/receivers a user is subscribed to.
#[derive(Debug, Default)]
pub(crate) struct HubUsers(HashMap<Uuid, HubSubscriptionInfo>);

impl HubUsers {
    /// Creates a new instance of `HubUsers`.
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    /// Subscribes a user to a specific channel.
    pub(crate) fn subscribe_user(&mut self, channel: &HubChannelName, hub_receiver: &HubReceiver) {
        let HubReceiver(user_id, receiver) = hub_receiver.resubscribe();
        let subscription_info = self.0.entry(user_id).or_default();

        subscription_info.insert(channel.clone(), receiver);
    }

    /// Unsubscribes a user from a specific channel.
    /// If the user is not subscribed to any other channels after this operation,
    /// they will be removed from the `HubUsers` collection.
    pub(crate) fn unsubscribe_user(&mut self, channel: &HubChannelName, user_id: Uuid) {
        if let Some(subscription_info) = self.0.get_mut(&user_id) {
            subscription_info.remove(channel);
            if subscription_info.is_empty() {
                self.0.remove(&user_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_user() {
        let mut hub_users = HubUsers::new();
        let channel = HubChannelName::try_from("test_channel").unwrap();
        let (_, receiver) = broadcast::channel(10);
        let user_id = Uuid::new_v4();
        let hub_receiver = HubReceiver(user_id, receiver);

        hub_users.subscribe_user(&channel, &hub_receiver);

        assert!(hub_users.0.contains_key(&user_id));
        assert!(hub_users.0.get(&user_id).unwrap().contains_key(&channel));
    }

    #[tokio::test]
    async fn test_unsubscribe_user() {
        let mut hub_users = HubUsers::new();
        let channel = HubChannelName::try_from("test_channel").unwrap();
        let (_, receiver) = broadcast::channel(10);
        let user_id = Uuid::new_v4();
        let hub_receiver = HubReceiver(user_id, receiver);

        hub_users.subscribe_user(&channel, &hub_receiver);
        hub_users.unsubscribe_user(&channel, user_id);

        assert!(!hub_users.0.contains_key(&user_id));
    }

    #[tokio::test]
    async fn test_unsubscribe_user_from_multiple_channels() {
        let mut hub_users = HubUsers::new();
        let channel1 = HubChannelName::try_from("test_channel1").unwrap();
        let channel2 = HubChannelName::try_from("test_channel2").unwrap();
        let (_, receiver1) = broadcast::channel(10);
        let (_, receiver2) = broadcast::channel(10);
        let user_id = Uuid::new_v4();
        let hub_receiver1 = HubReceiver(user_id, receiver1);
        let hub_receiver2 = HubReceiver(user_id, receiver2);

        hub_users.subscribe_user(&channel1, &hub_receiver1);
        hub_users.subscribe_user(&channel2, &hub_receiver2);
        hub_users.unsubscribe_user(&channel1, user_id);

        assert!(hub_users.0.contains_key(&user_id));
        assert!(!hub_users.0.get(&user_id).unwrap().contains_key(&channel1));
        assert!(hub_users.0.get(&user_id).unwrap().contains_key(&channel2));
    }
}
