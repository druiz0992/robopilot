use imu_common::types::untimed::{Scalar, UnitQuaternion, XYZ};
use log::info;
use notification_hub::models::hub::{HubChannelName, HubMessage};
use serde_json;
use std::io::{Error, ErrorKind};
use tokio::time::Duration;

use test_utils::hub;
use test_utils::ClientPipeOptions;

/// Example mimics a scenario with several data sources incomming from
/// different media. In this case, there are three sensor sources incoming from
///  "serial" port (orientation, odometry and distance). Additinally, there
/// is another data source from available from a WebsoSocker (Joystick controls).
///

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let pipe_options = vec![
        ClientPipeOptions::new(None, Some("/tmp/serial_pipe_3"), "distance", 1, 23, 500)
            .map_err(|e| Error::new(ErrorKind::Other, e))?,
        ClientPipeOptions::new(None, Some("/tmp/serial_pipe_2"), "orientation", 4, 100, 400)
            .map_err(|e| Error::new(ErrorKind::Other, e))?,
        ClientPipeOptions::new(None, Some("/tmp/serial_pipe_1"), "odometry", 2, 100, 345)
            .map_err(|e| Error::new(ErrorKind::Other, e))?,
        ClientPipeOptions::new(None, Some("/tmp/serial_pipe_4"), "joystick", 2, 456, 1000)
            .map_err(|e| Error::new(ErrorKind::Other, e))?,
    ];
    let mut hub = hub::start_hub(Some(pipe_options.clone()), None, None)
        .await
        .unwrap();
    let channels = hub::start_pipe_data_sources(pipe_options).await;

    info!(
        "############################### Waiting for channels: {:?} to become available",
        channels
    );
    // wait until all sensor channels from pipe and ws clients are available
    hub::wait_for_channels(&hub, &channels).await;

    // register to channels
    let hub_receivers = hub::register_to_channels(&mut hub, &channels).await;

    // process channels
    hub::listen_to_channel("odometry", &hub_receivers, Box::new(odometry_processor)).await;
    hub::listen_to_channel(
        "orientation",
        &hub_receivers,
        Box::new(orientation_processor),
    )
    .await;
    hub::listen_to_channel("distance", &hub_receivers, Box::new(distance_processor)).await;
    hub::listen_to_channel("joystick", &hub_receivers, Box::new(joystick_processor)).await;

    tokio::time::sleep(Duration::from_secs(50)).await;

    Ok(())
}

fn odometry_processor(channel: HubChannelName, message: HubMessage) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<XYZ>(data.as_str()) {
        println!(
            "Odometry processor received message {:?} from channel {:?}",
            sample, channel
        );
    }
}

fn orientation_processor(channel: HubChannelName, message: HubMessage) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<UnitQuaternion>(data.as_str()) {
        println!(
            "Orientation processor received message {:?} from channel {:?}",
            sample, channel
        );
    }
}

fn distance_processor(channel: HubChannelName, message: HubMessage) {
    let data = message.data.as_str();
    if let Ok(sample) = serde_json::from_str::<Scalar>(data) {
        println!(
            "Distance processor received message {:?} from channel {:?}",
            sample, channel
        );
    }
}

fn joystick_processor(channel: HubChannelName, message: HubMessage) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<XYZ>(data.as_str()) {
        println!(
            "Joystick processor received message {:?} from channel {:?}",
            sample, channel
        );
    }
}
