use imu_common::types::untimed::{Scalar, UnitQuaternion, XYZ};
use log::info;
use notification_hub::models::hub::{hub_message, HubChannelName, HubData, HubMessage};
use notification_hub::services::hub::HubManager;
use serde_json;
use std::future::Future;
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::sync::Mutex;

use test_utils::hub;

/// Example stats a hub with a serial and a web socket client. The serial port client connects
/// to port to /dev/ttyACM0 where there is a process  sending odomedry data. The web socket client
/// receives data from the frontend joystick.

const WS_IDX: usize = 0;
const SERIAL_IDX: usize = 1;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let serial_port_options = ("/dev/ttyACM0", 115200);
    let ws_url = "192.168.1.41:8080";

    let mut hub = hub::start_hub(None, Some(ws_url), Some(serial_port_options))
        .await
        .unwrap();
    let channels = [
        HubChannelName::try_from("lm393").unwrap(),   // odometry
        HubChannelName::try_from("hcsr04").unwrap(),  // distance
        HubChannelName::try_from("mpu6050").unwrap(), // orientation
        HubChannelName::try_from("mpu6050_accel").unwrap(), // orientation
        HubChannelName::try_from("joystick").unwrap(),
    ];
    let update_channel = HubChannelName::try_from("DEBUG").unwrap();

    // wait until all sensor channels from pipe and ws clients are available
    hub::wait_for_channels(&hub, &channels).await;

    // register to channels
    let hub_receivers = hub::register_to_channels(&mut hub, &channels).await;

    // Register to update channel
    let update_receiver = hub::register_to_channels(&mut hub, &[update_channel.clone()]).await;

    let hub = Arc::new(Mutex::new(hub));
    // process channels

    // debug
    hub::listen_to_channel(
        hub.clone(),
        update_channel.clone(),
        &update_receiver,
        hub::create_processor(update_processor),
    )
    .await;

    // joystick
    hub::listen_to_channel(
        hub.clone(),
        channels[4].clone(),
        &hub_receivers,
        hub::create_processor(joystick_processor),
    )
    .await;
    // odometry
    hub::listen_to_channel(
        hub.clone(),
        channels[0].clone(),
        &hub_receivers,
        hub::create_processor(odometry_processor),
    )
    .await;

    // orientation
    hub::listen_to_channel(
        hub.clone(),
        channels[2].clone(),
        &hub_receivers,
        hub::create_processor(orientation_processor),
    )
    .await;

    // accelerometer
    hub::listen_to_channel(
        hub.clone(),
        channels[3].clone(),
        &hub_receivers,
        hub::create_processor(acceleration_processor),
    )
    .await;

    println!("Press Ctrl+C to exit...");
    ctrl_c().await?;
    println!("Received Ctrl+C, shutting down.");

    Ok(())
}

async fn update_processor(
    _hub: Arc<Mutex<HubManager>>,
    channel: HubChannelName,
    message: HubMessage,
) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<XYZ>(data.as_str()) {
        info!(
            "##################### Update processor received message {:?} from channel {:?}, {:?}",
            sample, channel, data
        );
    }
}
async fn odometry_processor(
    _hub: Arc<Mutex<HubManager>>,
    channel: HubChannelName,
    message: HubMessage,
) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<XYZ>(data.as_str()) {
        info!(
            "Odometry processor received message {:?} from channel {:?}",
            sample, channel
        );
    }
}

async fn joystick_processor(
    hub: Arc<Mutex<HubManager>>,
    channel: HubChannelName,
    message: HubMessage,
) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<XYZ>(data.as_str()) {
        let new_data = format!("{:.2}, {:.2}", sample.0.x, sample.0.y)
            .parse::<HubData>()
            .unwrap();
        let new_channel = HubChannelName::try_from("ln298").unwrap();
        let hub_message = HubMessage::new(new_channel, new_data);
        info!(
            "Joystick processor received message {:?} from channel {:?} and converted to {:?}",
            sample, channel, hub_message
        );
        let hub_lock = hub.lock().await;

        let _ = hub_lock.send_to_client(hub_message, SERIAL_IDX).await;
    } else {
        info!("ERRORRORORORORO");
    }
}

async fn orientation_processor(
    hub: Arc<Mutex<HubManager>>,
    channel: HubChannelName,
    message: HubMessage,
) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<UnitQuaternion>(data.as_str()) {
        info!(
            "Orientation processor received {:?} from channel {:?}",
            sample, channel
        );
    }
}

async fn acceleration_processor(
    hub: Arc<Mutex<HubManager>>,
    channel: HubChannelName,
    message: HubMessage,
) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<XYZ>(data.as_str()) {
        info!(
            "Acceleration processor received {:?} from channel {:?}",
            sample, channel
        );
    }
}
