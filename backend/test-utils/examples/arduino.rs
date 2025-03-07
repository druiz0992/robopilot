use imu_common::types::untimed::XYZ;
use notification_hub::models::hub::{HubChannelName, HubMessage};
use serde_json;
use tokio::signal::ctrl_c;

use test_utils::hub;

/// Example stats a hub with a serial and a web socket client. The serial port client connects
/// to port to /dev/ttyACM0 where there is a process  sending odomedry data. The web socket client
/// receives data from the frontend joystick.

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let serial_port_options = ("/dev/ttyACM0", 9600);
    let ws_url = "192.168.1.69:8080";

    let mut hub = hub::start_hub(None, Some(ws_url), Some(serial_port_options))
        .await
        .unwrap();
    let channels = [
        HubChannelName::try_from("odometry").unwrap(),
        HubChannelName::try_from("joystick").unwrap(),
    ];

    // wait until all sensor channels from pipe and ws clients are available
    hub::wait_for_channels(&hub, &channels).await;

    // register to channels
    let hub_receivers = hub::register_to_channels(&mut hub, &channels).await;

    // process channels
    hub::listen_to_channel("odometry", &hub_receivers, Box::new(odometry_processor)).await;
    hub::listen_to_channel("joystick", &hub_receivers, Box::new(joystick_processor)).await;

    println!("Press Ctrl+C to exit...");
    ctrl_c().await?;
    println!("Received Ctrl+C, shutting down.");

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

fn joystick_processor(channel: HubChannelName, message: HubMessage) {
    let data = format!(r#""{}""#, message.data.as_str());
    if let Ok(sample) = serde_json::from_str::<XYZ>(data.as_str()) {
        println!(
            "Joystick processor received message {:?} from channel {:?}",
            sample, channel
        );
    }
}
