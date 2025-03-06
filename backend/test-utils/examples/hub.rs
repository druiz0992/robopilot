use notification_hub::models::hub::{HubChannelName, HubMessage};
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
    let path_pipes = vec![
        "/tmp/serial_pipe_1",
        "/tmp/serial_pipe_2",
        "/tmp/serial_pipe_3",
    ];
    let ws_url = "localhost:8080";

    let mut hub = hub::start_hub(Some(path_pipes.clone()), Some(ws_url))
        .await
        .unwrap();
    let pipe_channels = hub::start_pipe_data_sources(path_pipes).await;
    let ws_channels = hub::start_ws_data_sources(ws_url).await;
    let channels = [pipe_channels, ws_channels].concat();

    // wait until all sensor channels from pipe and ws clients are available
    hub::wait_for_channels(&hub, &channels).await;

    // register to channels
    let hub_receivers = hub::register_to_channels(&mut hub, &channels).await;

    // process channels
    hub::listen_to_channel("odometry", &hub_receivers, Box::new(default_processor)).await;
    hub::listen_to_channel("orientation", &hub_receivers, Box::new(default_processor)).await;
    hub::listen_to_channel("distance", &hub_receivers, Box::new(default_processor)).await;
    hub::listen_to_channel("joystick", &hub_receivers, Box::new(default_processor)).await;

    tokio::time::sleep(Duration::from_secs(50)).await;

    Ok(())
}

fn default_processor(channel: HubChannelName, message: HubMessage) {
    println!(
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXx Default processor received message {:?} from channel {:?}",
        message, channel
    );
}
