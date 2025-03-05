use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::{debug, error, info, warn};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::adapters::websocket::message::WsMessage;
use crate::models::hub::{HubChannelName, HubData};

type PeerMap = HashMap<SocketAddr, UnboundedSender<Message>>;
type ChannelMap = Arc<Mutex<HashMap<HubChannelName, PeerMap>>>;

/// WebSocket Server of Pub Sub Topic network
/// Server can receive 4 different WsMessages:
/// - WsMessage::Subscribe -> Server adds subscriber to topic channel
/// - WsMessage::Unsubscribe -> Server removes subcriber from topic channel
/// - WsMessage::Data -> Server broascasts message from topic to all registered
///   subscribers
/// - WsMessage::ListChannelsReq -> Responds with WsMessage::ListChannelsRep
///   containing available topic channels
#[derive(Debug)]
pub struct WebSocketServer {
    url: String,
    channel_map: ChannelMap,
}

impl WebSocketServer {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            channel_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start server
    pub async fn start(&self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(&self.url).await?;
        let channel_map = self.channel_map.clone(); // Clone the channel map
        info!("Listening on: {}", self.url);

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        tokio::spawn(handle_connection(channel_map.clone(), stream, addr));
                    }
                    Err(e) => {
                        warn!("Failed to accept connection: {:?}", e);
                    }
                }
            }
        });
        info!("WS server started");
        Ok(())
    }
}

// Handlers

/// WsMessage::Data handler. Broadcasts received data to all subscribers registered to channel
async fn handle_ws_data(
    channel_map: &ChannelMap,
    channel_name: &HubChannelName,
    data: HubData,
    addr: SocketAddr,
) {
    let mut channels = channel_map.lock().await;
    // Add new topic if necessary
    channels.entry(channel_name.clone()).or_insert_with({
        info!("New channel created: {:?}", channel_name);
        HashMap::new
    });

    // broadcast message to subscribers
    if let Some(subscribers) = channels.get(channel_name) {
        let ws_message = WsMessage::send_data_channel(channel_name.clone(), data)
            .to_string()
            .unwrap();

        info!(
            "Broadcasting message: {:?}  with subscribers {:?}",
            ws_message, subscribers
        );
        for (&peer_addr, peer_tx) in subscribers {
            if peer_addr != addr {
                debug!("Message sent to {:?}", addr);
                let _ = peer_tx.unbounded_send(Message::Text(ws_message.clone()));
            }
        }
    }
}

/// WsMessage::Subscribe handler. Registers new subscriber to channel
async fn handle_ws_subscribe(
    channel_map: &ChannelMap,
    channel_name: &HubChannelName,
    tx: UnboundedSender<Message>,
    addr: SocketAddr,
) {
    info!(
        "Subscription request to channel {:?} from {:?}",
        channel_name, addr
    );
    let mut channels = channel_map.lock().await;
    channels
        .entry(channel_name.clone())
        .or_default()
        .insert(addr, tx.clone());

    info!("Client {} subscribed to {:?}", addr, channel_name);
}

/// WsMessage::Unsubscribe handler. Deregisters new subscriber from channel
async fn handle_ws_unsubscribe(
    channel_map: &ChannelMap,
    channel_name: &HubChannelName,
    addr: SocketAddr,
) {
    info!(
        "Unsubscription request from channel {:?} from {:?}",
        channel_name, addr
    );
    let mut channels = channel_map.lock().await;
    if let Some(subscribers) = channels.get_mut(channel_name) {
        subscribers.remove(&addr);
        info!("Client {} unsubscribed from {:?}", addr, channel_name);
    }
}

/// WsMessage::ListChannelsReq handler. Sends requester a WsMessage::ListChannelsResp containing
/// the available topic channels
async fn handle_ws_list_channels(channel_map: &ChannelMap, tx: UnboundedSender<Message>) {
    let channels = channel_map.lock().await;
    let available_channels: Vec<HubChannelName> = channels.keys().cloned().collect();
    let ws_list_channels_resp = WsMessage::ListChannelsResponse(available_channels.clone());
    info!(
        "Received List Channels Request. Sending Response: {:?}",
        ws_list_channels_resp
    );
    let _ = tx.unbounded_send(Message::Text(ws_list_channels_resp.to_string().unwrap()));
}

/// Dispatches received message to handler
async fn handle_connection(channel_map: ChannelMap, raw_stream: TcpStream, addr: SocketAddr) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Websocket handshake failed: {:?}", e);
            return;
        }
    };
    info!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();
    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let msg_text = msg.to_text().unwrap_or_default().to_string();
        let channel_map = channel_map.clone();
        let tx = tx.clone();
        async move {
            match WsMessage::try_from(msg_text) {
                Ok(ws_message) => match ws_message {
                    WsMessage::Data(channel_name, data) => {
                        handle_ws_data(&channel_map, &channel_name, data, addr).await
                    }
                    WsMessage::ListChannelsReq => handle_ws_list_channels(&channel_map, tx).await,
                    WsMessage::Subscribe(channel_name) => {
                        handle_ws_subscribe(&channel_map, &channel_name, tx, addr).await
                    }
                    WsMessage::Unsubscribe(channel_name) => {
                        handle_ws_unsubscribe(&channel_map, &channel_name, addr).await
                    }
                    _ => warn!("Unknown WsMessage received"),
                },
                Err(e) => {
                    warn!("Unknown WsMessage received: {:?}", e);
                }
            }

            Ok(())
        }
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
    info!("{} disconnected", &addr);
    let mut channels = channel_map.lock().await;
    for subscribers in channels.values_mut() {
        subscribers.remove(&addr);
    }
}
