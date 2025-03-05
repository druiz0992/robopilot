use robopilot::adapters::serial::SerialClient;
use robopilot::adapters::websocket::WebSocketClient;
use robopilot::services::hub::HubManager;

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

    Ok(())
}
