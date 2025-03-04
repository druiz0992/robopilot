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
    if let Ok(serial_client) = SerialClient::new("/dev/ttyACM0", 9600) {
        let hub_serial = HubManager::new(serial_client);
        hub_serial.start().await;
    }
    let ws_client = WebSocketClient::new("localhost:8080").await.unwrap();
    let hub_ws = HubManager::new(ws_client);
    hub_ws.start().await;

    Ok(())
}
