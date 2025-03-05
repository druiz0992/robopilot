use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::adapters::websocket::WsMessage;
use crate::models::hub::{HubChannelName, HubMessage};

const TIMEOUT_SECS: u64 = 1;

pub(crate) async fn handle_incoming_data(
    sender: Arc<Mutex<broadcast::Sender<HubMessage>>>,
    incoming_data: HubMessage,
) {
    info!("Sending Data message {:?}", incoming_data);
    let sender_lock = sender.lock().await;
    let _ = sender_lock.send(incoming_data);
}

pub(crate) async fn handle_send_ws_message(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    message: WsMessage,
) -> Result<(), std::io::Error> {
    info!("Sending new WeMessage: {:?}", message);
    // Establish WebSocket connection
    let message_string = message.to_string().map_err(|e| {
        error!("Message conversion failed: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Conversion failed")
    })?;

    write
        .send(Message::Text(message_string))
        .await
        .map_err(|e| {
            error!("WebSocket send error: {:?}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "WebSocket send failed")
        })?;

    Ok(())
}

pub(crate) async fn handle_send_ws_message_with_response(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    message: WsMessage,
) -> Result<Vec<HubChannelName>, std::io::Error> {
    handle_send_ws_message(write, message).await?;

    let timeout_duration = std::time::Duration::from_secs(TIMEOUT_SECS);
    //let mut receiver = response.lock().await;

    tokio::select! {
        resp = read.next() => {
            match resp {
                Some(Ok(Message::Text(resp_text))) => {
                    info!("Received response: {:?}", resp_text);
                    if let Ok(WsMessage::ListChannelsResponse(channels)) = WsMessage::try_from(resp_text) {
                        return Ok(channels);
                    }
                    warn!("Received unexpected Websocket message type");
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unexpected response"))
                }
                Some(Ok(_)) => {
                    warn!("Received unexpected Websocket message format");
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unexpected response"))
                }
                Some(Err(e)) => {
                    error!("Websocket read error: {:?}",e);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "Websocket read failed"))
                }
                None => {
                    warn!("Websocket connection closed before response received");
                    Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Websocket closed"))
                }
            }
        }
        _ = tokio::time::sleep(timeout_duration) => {
            warn!("Timeout waiting for WsMessage response");
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Response timed out"))
        }
    }
}
