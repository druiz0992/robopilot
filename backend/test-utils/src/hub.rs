use log::info;
use notification_hub::adapters::serial::SerialClient;
use notification_hub::adapters::websocket::WebSocketClient;
use notification_hub::models::hub::{HubChannelName, HubMessage};
use notification_hub::services::hub::controller::HubReceiver;
use notification_hub::services::hub::HubManager;
use std::collections::HashMap;
use tokio::time::Duration;

use crate::ClientPipeOptions;
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
        info!("Registered to channel {:?}", channel.clone());
    }
    hub_receiver
}

pub async fn wait_for_channels(hub: &HubManager, channels: &[HubChannelName]) {
    loop {
        let available_channels: Vec<_> = hub.list_channels().await.unwrap().into_iter().collect();
        info!(
            "Available channels: {:?}, Requested channels: {:?}",
            available_channels, channels
        );
        let all_available = channels
            .iter()
            .all(|item| available_channels.contains(item));

        if all_available {
            break;
        }
        tokio::time::sleep(Duration::from_millis(900)).await;
    }
}

pub async fn start_hub(
    pipe_options: Option<Vec<ClientPipeOptions>>,
    ws_url: Option<&str>,
    serial_port_options: Option<(&str, u32)>,
) -> Result<HubManager, std::io::Error> {
    let mut hub = HubManager::new();
    if let Some(pipe_options) = pipe_options {
        let pipe_read_path: Vec<_> = pipe_options
            .into_iter()
            .filter_map(|o| o.read_path())
            .collect();
        let pipe_read_path: Vec<_> = pipe_read_path.iter().map(|s| s.as_str()).collect();
        if let Ok(pipe_client) = PipeClient::new(None, Some(pipe_read_path)).await {
            hub.add(Box::new(pipe_client));
        }
    }
    if let Some(ws_url) = ws_url {
        if let Ok(ws_client) = WebSocketClient::new(ws_url).await {
            hub.add(Box::new(ws_client));
        }
    }
    if let Some(serial_port_options) = serial_port_options {
        if let Ok(serial_client) = SerialClient::new(serial_port_options.0, serial_port_options.1) {
            hub.add(Box::new(serial_client));
        }
    }

    hub.start().await?;
    Ok(hub)
}

pub async fn start_pipe_data_sources(options: Vec<ClientPipeOptions>) -> Vec<HubChannelName> {
    let mut channels = Vec::new();

    let distance_channel = options[0].channel();
    let orientation_channel = options[1].channel();
    let odometry_channel = options[2].channel();
    let joystick_channel = options[3].channel();

    channels.push(distance_channel.clone());
    channels.push(orientation_channel.clone());
    channels.push(odometry_channel.clone());
    channels.push(joystick_channel.clone());

    let distance_path = options[0].read_path();
    let distance_dims = options[0].n_dims();
    let distance_delay = options[0].delay();
    let distance_period = options[0].period();
    let distance_client = PipeClient::new(distance_path.as_deref(), None)
        .await
        .unwrap();
    let mut distance = DataSource::new(distance_client, distance_dims, distance_channel);

    let orientation_path = options[1].read_path();
    let orientation_dims = options[1].n_dims();
    let orientation_delay = options[1].delay();
    let orientation_period = options[1].period();
    let orientation_client = PipeClient::new(orientation_path.as_deref(), None)
        .await
        .unwrap();
    let mut orientation =
        DataSource::new(orientation_client, orientation_dims, orientation_channel);

    let odometry_path = options[2].read_path();
    let odometry_dims = options[2].n_dims();
    let odometry_delay = options[2].delay();
    let odometry_period = options[2].period();
    let odometry_client = PipeClient::new(odometry_path.as_deref(), None)
        .await
        .unwrap();
    let mut odometry = DataSource::new(odometry_client, odometry_dims, odometry_channel);

    let joystick_path = options[3].read_path();
    let joystick_dims = options[3].n_dims();
    let joystick_delay = options[3].delay();
    let joystick_period = options[3].period();
    let joystick_client = PipeClient::new(joystick_path.as_deref(), None)
        .await
        .unwrap();
    let mut joystick = DataSource::new(joystick_client, joystick_dims, joystick_channel);

    tokio::spawn(async move {
        tokio::try_join!(
            odometry.start(odometry_delay, odometry_period),
            distance.start(distance_delay, distance_period),
            orientation.start(orientation_delay, orientation_period),
            joystick.start(joystick_delay, joystick_period),
        )
        .unwrap();
    });

    tokio::time::sleep(Duration::from_secs(3)).await;
    channels
}
