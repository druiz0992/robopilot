use notification_hub::adapters::websocket::WebSocketClient;
use notification_hub::models::hub::{HubChannelName, HubMessage};
use notification_hub::services::hub::controller::HubReceiver;
use notification_hub::services::hub::HubManager;
use std::collections::HashMap;
use tokio::time::Duration;

use crate::DataSource;
use crate::PipeClient;

pub type RegisteredReceivers = HashMap<HubChannelName, HubReceiver>;
pub type ProcessorFunction = Box<dyn Fn(HubChannelName, HubMessage) + Send + Sync>;

pub async fn listen_to_channel(
    channel_str: &str,
    receivers: &RegisteredReceivers,
    processor: ProcessorFunction,
) {
    let channel = HubChannelName::try_from(channel_str).unwrap();
    let hub_receiver = receivers.get(&channel).unwrap();
    let mut receiver = hub_receiver.receiver();
    tokio::spawn(async move {
        loop {
            if let Ok(data) = receiver.recv().await {
                processor(channel.clone(), data);
            }
        }
    });
}

pub async fn register_to_channels(
    hub: &mut HubManager,
    channels: &[HubChannelName],
) -> RegisteredReceivers {
    let mut hub_receiver = HashMap::new();
    for channel in channels {
        hub_receiver.insert(
            channel.clone(),
            hub.register_to_channel(channel.clone()).await.unwrap(),
        );
    }
    hub_receiver
}

pub async fn wait_for_channels(hub: &HubManager, channels: &[HubChannelName]) {
    loop {
        let available_channels: Vec<_> = hub.list_channels().await.unwrap().into_iter().collect();
        if available_channels == channels {
            break;
        }
        tokio::time::sleep(Duration::from_millis(900)).await;
    }
}

pub async fn start_hub(
    path_pipes: Option<Vec<&str>>,
    ws_url: Option<&str>,
) -> Result<HubManager, std::io::Error> {
    let mut hub = HubManager::new();
    if let Some(path_pipes) = path_pipes {
        if let Ok(pipe_client) = PipeClient::new(None, Some(path_pipes)).await {
            hub.add(Box::new(pipe_client));
        }
    }
    if let Some(ws_url) = ws_url {
        if let Ok(ws_client) = WebSocketClient::new(ws_url).await {
            hub.add(Box::new(ws_client));
        }
    }

    hub.start().await?;
    Ok(hub)
}

pub async fn start_pipe_data_sources(path_pipes: Vec<&str>) -> Vec<HubChannelName> {
    let mut channels = Vec::new();
    let distance_channel = HubChannelName::try_from("Distance").unwrap();
    let orientation_channel = HubChannelName::try_from("Orientation").unwrap();
    let odometry_channel = HubChannelName::try_from("Odometry").unwrap();
    channels.push(distance_channel.clone());
    channels.push(orientation_channel.clone());
    channels.push(odometry_channel.clone());

    let distance_client = PipeClient::new(Some(path_pipes[0]), None).await.unwrap();
    let mut distance = DataSource::new(distance_client, 1, distance_channel);

    let orientation_client = PipeClient::new(Some(path_pipes[1]), None).await.unwrap();
    let mut orientation = DataSource::new(orientation_client, 4, orientation_channel);

    let odometry_client = PipeClient::new(Some(path_pipes[2]), None).await.unwrap();
    let mut odometry = DataSource::new(odometry_client, 2, odometry_channel);

    tokio::spawn(async move {
        tokio::try_join!(
            odometry.start(1),
            distance.start(200),
            orientation.start(500),
        )
        .unwrap();
    });

    tokio::time::sleep(Duration::from_secs(3)).await;
    channels
}

pub async fn start_ws_data_sources(url: &str) -> Vec<HubChannelName> {
    let mut channels = Vec::new();
    let joystick_channel = HubChannelName::try_from("Joystick").unwrap();
    channels.push(joystick_channel.clone());

    let joystick_client = WebSocketClient::new(url).await.unwrap();
    let mut joystick = DataSource::new(joystick_client, 2, joystick_channel);

    tokio::spawn(async move {
        tokio::try_join!(joystick.start(102)).unwrap();
    });

    tokio::time::sleep(Duration::from_secs(3)).await;
    channels
}
