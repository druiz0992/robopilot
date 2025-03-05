use log::{error, info};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use super::channel::HubChannels;
use super::user::HubUsers;
use crate::models::hub::{HubChannelName, HubMessage};
use crate::ports::NotificationHub;

const CHANNEL_CAPACITY: usize = 100;

/// Tuple cosisting of user id (Uuid) and channel receiver
#[derive(Debug)]
pub struct HubReceiver(pub Uuid, pub(crate) broadcast::Receiver<HubMessage>);

impl HubReceiver {
    pub(crate) fn resubscribe(&self) -> Self {
        let receiver = self.1.resubscribe();
        Self(self.0, receiver)
    }
    pub fn user_id(&self) -> Uuid {
        self.0
    }
    pub fn receiver(&self) -> broadcast::Receiver<HubMessage> {
        self.1.resubscribe()
    }
}

/// `HubManager` controls communications through a NotificationHub network by
/// maintaining the set of topic channels in the hub, the set of subscribers
/// to specific topic channels, and ensuring that subscribers receive
/// published information.
/// `HubManager` contains a number of hub nodes, which implement NotificationHub.
/// These nodes mimic a pub sub network, where one can subscribe to a given topic channel.
/// Hub_sender and hub_receiver are the sender and receiver channels where the HubManager
/// receives information from hub nodes. This information is then dispatched to subscribed
/// users
///
/// A subscriber is typically a processing entity that wants to receive certain
///  data from the hub. For example, a control unit that needs to compute the path
/// from A to B in an autonomous robot would subscribe to certain channels containing
///  relevant sensor information (position, camera, odometry...). Once subscribed,
/// sensor data is available through the receiver channel in the form of `HubMessages`

#[derive(Debug)]
pub struct HubManager<H: NotificationHub> {
    channels: Arc<Mutex<HubChannels>>,
    subscribers: HubUsers,
    hub_sender: broadcast::Sender<HubMessage>,
    hub_receiver: Arc<Mutex<broadcast::Receiver<HubMessage>>>,
    hub_node: H,
}

impl<H: NotificationHub> HubManager<H> {
    pub fn new(hub_node: H) -> Self {
        let (hub_sender, hub_receiver) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            channels: Arc::new(Mutex::new(HubChannels::new())),
            subscribers: HubUsers::new(),
            hub_sender,
            hub_receiver: Arc::new(Mutex::new(hub_receiver)),
            hub_node,
        }
    }

    /// Request hub node to register to specific channel
    async fn register_to_hub_channel(
        &self,
        channel: &HubChannelName,
    ) -> Result<(), std::io::Error> {
        self.hub_node.subscribe(channel.clone()).await
    }

    // Request hub node to unregister from speficic channel
    async fn unregister_from_hub_channel(
        &self,
        channel: &HubChannelName,
    ) -> Result<(), std::io::Error> {
        self.hub_node.unsubscribe(channel.clone()).await
    }

    // Start hub.
    pub async fn start(&self) -> Result<(), std::io::Error> {
        let hub_sender = self.hub_sender.clone();
        self.hub_node.start(hub_sender).await?;
        let hub_receiver = self.hub_receiver.clone();
        let channels = self.channels.clone();

        tokio::spawn(async move {
            let mut receiver = hub_receiver.lock().await;
            while let Ok(data) = receiver.recv().await {
                // retrieve channel from data and broadcast to all registered clients
                info!("Received data: {:?}", data);
                let channels_lock = channels.lock().await;
                if let Some(sender) = channels_lock.get_sender(&data.channel) {
                    let _ = sender.send(data).map_err(|e| error!("Error : {:?}", e));
                }
            }
        });
        Ok(())
    }

    // List availabe topic channels in the Hub network
    pub async fn list_channels(&self) -> Result<Vec<HubChannelName>, std::io::Error> {
        self.hub_node.list_channels().await
    }

    // Returns a receiver channel for a specific channel that the requestor can listen to
    // to obtain data from a topic channel
    pub async fn register_to_channel(
        &mut self,
        channel: HubChannelName,
    ) -> Result<HubReceiver, std::io::Error> {
        // subscribe user to channel
        let mut channels = self.channels.lock().await;
        let receiver = channels.subscribe_user(&channel);
        self.subscribers.subscribe_user(&channel, &receiver);
        if channels.get_number_subscribers(&channel) == 1 {
            self.register_to_hub_channel(&channel).await?;
        }
        Ok(receiver.resubscribe())
    }

    // Unsubscribes from topic channel
    pub async fn unregister_from_channel(
        &mut self,
        channel: HubChannelName,
        user_id: Uuid,
    ) -> Result<(), std::io::Error> {
        let mut channels = self.channels.lock().await;
        channels.unsubscribe_user(&channel, user_id);
        self.subscribers.unsubscribe_user(&channel, user_id);
        if channels.is_empty(&channel) {
            self.unregister_from_hub_channel(&channel).await?;
        }
        Ok(())
    }

    // Send HubMessage to topic channel.
    pub async fn send_to_channel(&self, message: HubMessage) -> Result<(), std::io::Error> {
        self.hub_node.send(message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::websocket::WebSocketClient;
    use crate::models::hub::HubData;

    const URL: &str = "localhost:8080";

    #[tokio::test]
    async fn test_wsocket() {
        let _ = env_logger::builder().is_test(true).try_init();

        let client1 = WebSocketClient::new(URL).await.unwrap();
        let client2 = WebSocketClient::new(URL).await.unwrap();
        let hub_ws1 = HubManager::new(client1);
        let mut hub_ws2 = HubManager::new(client2);
        hub_ws1.start().await.unwrap();
        hub_ws2.start().await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // check channels are empty
        info!("###################  List channels");
        let channels = hub_ws1.list_channels().await.unwrap();
        assert!(channels.is_empty());

        // Send message to topic1. This will create a new channel
        info!("###################  Send first message to empty subscription list. This will create new channel");
        let ws_data = HubMessage::try_from_str("topic1", "test topic1").unwrap();
        hub_ws1.send_to_channel(ws_data).await.unwrap();

        let channels = hub_ws1.list_channels().await.unwrap();
        assert_eq!(channels, vec![HubChannelName::try_from("topic1").unwrap()]);

        // send message to topic1. Check that only topic1 is an active channel
        info!("###################  Send message to empty subscription list to existing channel");
        let ws_data = HubMessage::try_from_str("topic1", "test topic1").unwrap();
        hub_ws1.send_to_channel(ws_data).await.unwrap();

        let channels = hub_ws1.list_channels().await.unwrap();
        assert_eq!(channels, vec![HubChannelName::try_from("topic1").unwrap()]);

        // subscribe to channel topic 1 and send message
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        info!("##################  Subscribe to channel and send new message");
        let receiver = hub_ws2
            .register_to_channel(HubChannelName::try_from("topic1").unwrap())
            .await
            .unwrap();
        let user_id = receiver.user_id();

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        let ws_data = HubMessage::try_from_str("topic1", "new message test topic1").unwrap();
        hub_ws1.send_to_channel(ws_data).await.unwrap();

        let receiver = Arc::new(receiver);
        let receiver_clone = Arc::clone(&receiver);
        tokio::spawn(async move {
            let mut receiver = receiver_clone.receiver();
            if let Ok(msg) = receiver.recv().await {
                assert_eq!(msg.channel, HubChannelName::try_from("topic1").unwrap());
                assert_eq!(
                    msg.data,
                    "new message test topic1".parse::<HubData>().unwrap()
                );
            }
        })
        .await
        .unwrap();

        // unsubscribe to channel topic 1 and send message
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        info!("##################  UnSubscribe from channel and send new message");
        hub_ws2
            .unregister_from_channel(HubChannelName::try_from("topic1").unwrap(), user_id)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        let ws_data = HubMessage::try_from_str("topic1", "new message test topic1").unwrap();
        hub_ws1.send_to_channel(ws_data).await.unwrap();

        tokio::spawn(async move {
            let receiver = Arc::clone(&receiver);
            let mut receiver = receiver.receiver();
            if receiver.recv().await.is_ok() {
                panic!("Data shouldn't be available after unregister")
            }
        })
        .await
        .unwrap();
    }
}
