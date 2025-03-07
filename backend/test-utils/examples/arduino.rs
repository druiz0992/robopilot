use imu_common::types::timed::Sample3D;
use imu_common::types::untimed::XYZ;
use notification_hub::models::hub::{HubChannelName, HubMessage};
use serde_json;
use tokio::time::Duration;

use test_utils::hub;

/// Example mimics a scenario with several data sources incomming from
/// different media. In this case, there are three sensor sources incoming from
///  "serial" port (orientation, odometry and distance). Additinally, there
/// is another data source from available from a WebsoSocker (Joystick controls).
///

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let serial_port_options = ("/dev/ttyACM0", 9600);

    let mut hub = hub::start_hub(None, None, Some(serial_port_options))
        .await
        .unwrap();
    let channels = [HubChannelName::try_from("odometry").unwrap()];

    // wait until all sensor channels from pipe and ws clients are available
    hub::wait_for_channels(&hub, &channels).await;

    // register to channels
    let hub_receivers = hub::register_to_channels(&mut hub, &channels).await;

    // process channels
    hub::listen_to_channel("odometry", &hub_receivers, Box::new(odometry_processor)).await;

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
    if let Ok(sample) = serde_json::from_str::<Sample3D>(data.as_str()) {
        println!(
            "Odometry processor received message {:?} from channel {:?}",
            sample, channel
        );
    }
}
