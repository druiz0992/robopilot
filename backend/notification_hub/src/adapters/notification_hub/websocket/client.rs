use async_trait::async_trait;
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::models::hub::{HubChannelName, HubMessage};
use crate::ports::NotificationHub;

use super::handlers;
use super::message::WsMessage;
use super::server::WebSocketServer;

type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// `WebSocketClient` manages a bidirectional WebSocket connection.
/// It reads messages from the WebSocket and broadcasts them to subscribers.
#[derive(Debug, Clone)]
pub struct WebSocketClient {
    client_url: String,
    ws_write: Arc<Mutex<WsWrite>>,
    ws_read: Arc<Mutex<WsRead>>,
}

impl WebSocketClient {
    // Constructor to initialize WebSocketClient with a URL and broadcast channels
    pub async fn new(url: &str) -> Result<Self, std::io::Error> {
        let client_url = format!("ws://{}", url);

        // Launch server will fail it its already launched. Not very nice
        let _ = launch_server(url).await;

        match connect_async(client_url.as_str()).await {
            Ok((ws_stream, _)) => {
                let (write, read) = ws_stream.split();
                info!("Connected to WebSocket server at {}", client_url);
                Ok(Self {
                    client_url,
                    ws_write: Arc::new(Mutex::new(write)),
                    ws_read: Arc::new(Mutex::new(read)),
                })
            }

            Err(e) => {
                error!("WebSocket connection failed: {:?}", e);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Connection failed",
                ))
            }
        }
    }
}

async fn launch_server(url: &str) -> Result<(), std::io::Error> {
    let server = WebSocketServer::new(url);
    server.start().await
}

#[async_trait]
impl NotificationHub for WebSocketClient {
    // Send data to the WebSocket server
    async fn send(&self, data: HubMessage) -> Result<(), std::io::Error> {
        let ws_message = WsMessage::from(data);
        let mut ws_write = self.ws_write.lock().await;
        let _ = handlers::handle_send_ws_message(&mut ws_write, ws_message).await?;
        Ok(())
    }

    async fn list_channels(&self) -> Result<Vec<HubChannelName>, std::io::Error> {
        let ws_message = WsMessage::list_channels_req();
        info!("Sending List channels request message...");
        let (ws_stream, _) = connect_async(self.client_url.as_str())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        handlers::handle_send_ws_message_with_response(&mut ws_write, &mut ws_read, ws_message)
            .await
    }

    // Connect to the WebSocket server, receive messages, and handle them
    async fn start(
        &self,
        sender: Option<broadcast::Sender<HubMessage>>,
    ) -> Result<(), std::io::Error> {
        if let Some(sender) = sender {
            let sender = Arc::new(Mutex::new(sender));
            let sender_clone = sender.clone();
            tokio::spawn({
                let ws_read = Arc::clone(&self.ws_read);
                async move {
                    let mut stream = ws_read.lock().await;
                    while let Some(message) = stream.next().await {
                        match message {
                            Ok(Message::Text(text)) => {
                                // When a text message is received, handle it
                                info!("Received message from server: {}", text);
                                match WsMessage::try_from(text) {
                                    Ok(ws_message) => match ws_message {
                                        WsMessage::Data(channel, data) => {
                                            let hub_message = HubMessage::new(channel, data);
                                            handlers::handle_incoming_data(
                                                Arc::clone(&sender_clone),
                                                hub_message,
                                            )
                                            .await;
                                        }
                                        _ => {
                                            warn!("Unexpexted WsMessage received")
                                        }
                                    },
                                    Err(e) => {
                                        error!("Error in conversion: {}", e)
                                    }
                                }
                            }
                            Ok(m) => warn!("Unknown wsMessage type: {:?}", m),
                            Err(e) => {
                                error!("Error reading WebSocket message: {:?}", e);
                                break;
                            }
                        }
                    }
                    info!("WebSocket connection lost! Reconnecting...");
                }
            });
        }
        Ok(())
    }

    async fn subscribe(&self, channel: HubChannelName) -> Result<(), std::io::Error> {
        let ws_message = WsMessage::subscribe_channel(channel);
        info!("Send Subscription request: {:?}", ws_message);
        let mut ws_write = self.ws_write.lock().await;
        if let Err(e) = handlers::handle_send_ws_message(&mut ws_write, ws_message).await {
            error!("Failed to send subscribe message: {:?}", e);
        }
        Ok(())
    }

    async fn unsubscribe(&self, channel: HubChannelName) -> Result<(), std::io::Error> {
        let ws_message = WsMessage::unsubscribe_channel(channel);
        let mut ws_write = self.ws_write.lock().await;
        if let Err(e) = handlers::handle_send_ws_message(&mut ws_write, ws_message).await {
            error!("Failed to send unsubscribe message: {:?}", e);
        }
        Ok(())
    }
}
