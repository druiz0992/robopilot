use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::models::hub::{HubChannelName, HubMessage};

#[async_trait]
pub trait NotificationHub: Send + Sync {
    async fn send(&self, data: HubMessage) -> Result<(), std::io::Error>;
    async fn start(&self, sender: broadcast::Sender<HubMessage>) -> Result<(), std::io::Error>;
    async fn list_channels(&self) -> Result<Vec<HubChannelName>, std::io::Error>;
    async fn subscribe(&self, channel: HubChannelName) -> Result<(), std::io::Error>;
    async fn unsubscribe(&self, channel: HubChannelName) -> Result<(), std::io::Error>;
}
