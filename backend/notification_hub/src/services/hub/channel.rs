use std::collections::{HashMap, HashSet};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::controller::HubReceiver;
use crate::models::hub::{HubChannelName, HubMessage};

const CHANNEL_CAPACITY: usize = 100;

/// `HubChanelInfo` holds information about a channel, including
/// associated sender channel and the set of subscribed users
#[derive(Debug)]
struct HubChannelInfo {
    sender: broadcast::Sender<HubMessage>,
    subscribers: HashSet<Uuid>,
}

/// `HubChannels` manages the available hub channels identified by their name.
/// Each channel has an associated sender  and set of subscribers UUIDs.
#[derive(Debug)]
pub(crate) struct HubChannels(HashMap<HubChannelName, HubChannelInfo>);

impl HubChannels {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    // Subscribe new user to channel. Returns a HubReceiver consisting of
    //  newly associated user ID and receiver channel.
    pub(crate) fn subscribe_user(&mut self, channel: &HubChannelName) -> HubReceiver {
        let channel_info = self
            .0
            .entry(channel.clone())
            .or_insert_with(|| HubChannelInfo {
                sender: broadcast::channel(CHANNEL_CAPACITY).0,
                subscribers: HashSet::new(),
            });
        let user_id = Uuid::new_v4();
        channel_info.subscribers.insert(user_id);
        let receiver = channel_info.sender.subscribe();
        HubReceiver(user_id, receiver)
    }

    // Unsubscribe user identified by user_id from channel. If channel doesnt have
    // any additional subscrobers, channel is removed from `HubChannels`
    pub(crate) fn unsubscribe_user(&mut self, channel: &HubChannelName, user_id: Uuid) {
        if let Some(channel_info) = self.0.get_mut(channel) {
            channel_info.subscribers.remove(&user_id);
            if self.is_empty(channel) {
                self.0.remove(channel);
            }
        }
    }

    // Returns number of subscribers in a given channel
    pub(crate) fn get_number_subscribers(&self, channel: &HubChannelName) -> usize {
        if let Some(channel_info) = self.0.get(channel) {
            return channel_info.subscribers.len();
        }
        0
    }

    // Returns true if there are no subscribers in a given channel
    pub(crate) fn is_empty(&self, channel: &HubChannelName) -> bool {
        if let Some(channel_info) = self.0.get(channel) {
            return channel_info.subscribers.is_empty();
        }
        true
    }

    // Returns  the channel sender associated to a hub channel
    pub(crate) fn get_sender(
        &self,
        channel: &HubChannelName,
    ) -> Option<broadcast::Sender<HubMessage>> {
        if let Some(channel_info) = self.0.get(channel) {
            return Some(channel_info.sender.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hub_channels() {
        let hub_channels = HubChannels::new();
        assert!(hub_channels.0.is_empty());
    }

    #[test]
    fn test_subscribe_user() {
        let mut hub_channels = HubChannels::new();
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let hub_receiver = hub_channels.subscribe_user(&channel_name);

        assert_eq!(hub_channels.get_number_subscribers(&channel_name), 1);
        assert!(hub_channels.0.contains_key(&channel_name));
        assert!(hub_channels.0[&channel_name]
            .subscribers
            .contains(&hub_receiver.0));
    }

    #[test]
    fn test_unsubscribe_user() {
        let mut hub_channels = HubChannels::new();
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let hub_receiver = hub_channels.subscribe_user(&channel_name);

        hub_channels.unsubscribe_user(&channel_name, hub_receiver.0);
        assert_eq!(hub_channels.get_number_subscribers(&channel_name), 0);
        assert!(!hub_channels.0.contains_key(&channel_name));
    }

    #[test]
    fn test_get_number_subscribers() {
        let mut hub_channels = HubChannels::new();
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        hub_channels.subscribe_user(&channel_name);
        hub_channels.subscribe_user(&channel_name);

        assert_eq!(hub_channels.get_number_subscribers(&channel_name), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut hub_channels = HubChannels::new();
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        assert!(hub_channels.is_empty(&channel_name));

        hub_channels.subscribe_user(&channel_name);
        assert!(!hub_channels.is_empty(&channel_name));
    }

    #[test]
    fn test_get_sender() {
        let mut hub_channels = HubChannels::new();
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        hub_channels.subscribe_user(&channel_name);

        let sender = hub_channels.get_sender(&channel_name);
        assert!(sender.is_some());
    }
}
