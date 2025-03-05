use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::models::hub::{HubChannelName, HubMessage};

/// A trait representing a notification hub that can send and manage messages across different channels.

#[async_trait]
pub trait NotificationHub: Send + Sync + std::fmt::Debug {
    /// Sends a message to the notification hub.
    async fn send(&self, data: HubMessage) -> Result<(), std::io::Error>;
    /// Starts the notification hub with the given sender.
    async fn start(&self, sender: broadcast::Sender<HubMessage>) -> Result<(), std::io::Error>;
    /// Lists all available channels in the notification hub.
    async fn list_channels(&self) -> Result<Vec<HubChannelName>, std::io::Error>;
    /// Subscribes to a specific channel in the notification hub.
    async fn subscribe(&self, channel: HubChannelName) -> Result<(), std::io::Error> {
        Ok(())
    }
    /// Unsubscribes from a specific channel in the notification hub.
    async fn unsubscribe(&self, channel: HubChannelName) -> Result<(), std::io::Error> {
        Ok(())
    }
}
