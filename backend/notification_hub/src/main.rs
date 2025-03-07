use notification_hub::adapters::serial::SerialClient;
use notification_hub::adapters::websocket::WebSocketClient;
use notification_hub::services::hub::HubManager;

use tokio::signal::ctrl_c;

mod adapters;
mod models;
mod ports;
mod services;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let mut hub = HubManager::new();
    if let Ok(serial_client) = SerialClient::new("/dev/ttyACM0", 9600) {
        hub.add(Box::new(serial_client));
    }
    if let Ok(ws_client) = WebSocketClient::new("localhost:8080").await {
        hub.add(Box::new(ws_client));
    }

    hub.start().await?;

    println!("Press Ctrl+C to exit...");
    ctrl_c().await?;
    println!("Received Ctrl+C, shutting down.");

    Ok(())
}
