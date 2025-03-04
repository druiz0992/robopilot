use std::collections::{HashMap, HashSet};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::controller::HubReceiver;
use crate::models::hub::{HubChannelName, HubMessage};

#[derive(Debug)]
struct HubUserInfo {
    receiver: broadcast::Receiver<HubMessage>,
    subscribed_channels: HashSet<HubChannelName>,
}

#[derive(Debug)]
pub(crate) struct HubUsers(HashMap<Uuid, HubUserInfo>);

impl HubUsers {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn subscribe_user(&mut self, channel: &HubChannelName, hub_receiver: &HubReceiver) {
        let HubReceiver(user_id, receiver) = hub_receiver.resubscribe();
        let user_info = self.0.entry(user_id).or_insert_with(|| HubUserInfo {
            receiver,
            subscribed_channels: HashSet::new(),
        });

        user_info.subscribed_channels.insert(channel.clone());
    }

    pub(crate) fn unsubscribe_user(&mut self, channel: &HubChannelName, user_id: Uuid) {
        if let Some(user_info) = self.0.get_mut(&user_id) {
            user_info.subscribed_channels.remove(channel);
            if user_info.subscribed_channels.is_empty() {
                self.0.remove(&user_id);
            }
        }
    }
}
