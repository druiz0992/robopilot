use actix::dev::channel;
use std::collections::{HashMap, HashSet};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::controller::HubReceiver;
use crate::models::hub::{HubChannelName, HubMessage};

#[derive(Debug)]
struct HubChannelInfo {
    sender: broadcast::Sender<HubMessage>,
    subscribers: HashSet<Uuid>,
}

#[derive(Debug)]
pub(crate) struct HubChannel(HashMap<HubChannelName, HubChannelInfo>);

impl HubChannel {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn subscribe_user(&mut self, channel: &HubChannelName) -> HubReceiver {
        let channel_info = self
            .0
            .entry(channel.clone())
            .or_insert_with(|| HubChannelInfo {
                sender: broadcast::channel(100).0,
                subscribers: HashSet::new(),
            });
        let user_id = Uuid::new_v4();
        channel_info.subscribers.insert(user_id);
        let receiver = channel_info.sender.subscribe();
        HubReceiver(user_id, receiver)
    }

    pub(crate) fn unsubscribe_user(&mut self, channel: &HubChannelName, user_id: Uuid) {
        if let Some(channel_info) = self.0.get_mut(channel) {
            channel_info.subscribers.remove(&user_id);
            if channel_info.subscribers.is_empty() {
                self.0.remove(channel);
            }
        }
    }

    pub(crate) fn get_number_subscribers(&self, channel: &HubChannelName) -> usize {
        if let Some(channel_info) = self.0.get(channel) {
            return channel_info.subscribers.len();
        }
        0
    }

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
