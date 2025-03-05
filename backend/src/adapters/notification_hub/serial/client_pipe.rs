use async_trait::async_trait;
use log::{error, info, warn};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_serial::{DataBits, Parity, SerialPortBuilderExt, SerialStream, StopBits};

use super::channels::{SerialChannelName, SerialPubChannels};
use super::message::SerialRawMessage;
use crate::models::hub::{HubChannelName, HubMessage};
use crate::ports::NotificationHub;

const BUFFER_SIZE: usize = 1024;

/// The `SerialClient` struct represents a client that communicates with a serial port that can subscribe
/// to specific topic channels. It allows to send and receive messages on specific topics.
///
/// SerialClient holds a reference to the serial port and the topic channels
///
/// # Fields
/// - `port`: An `Arc<RwLock<SerialStream>>` that represents the serial port.
/// - `serial_channels`: An `Arc<RwLock<SerialPubChannels>>` that holds the topic channels.

#[derive(Debug)]
pub struct PipeClient {
    write_pipe: Arc<Mutex<File>>,
    read_pipe: Arc<Mutex<File>>,
    channels: Arc<RwLock<SerialPubChannels>>,
}

impl PipeClient {
    pub async fn new(path: &str) -> Result<Self, std::io::Error> {
        info!("Opening bidirectional pipe...");
        let write_pipe = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .await?;
        let read_pipe = OpenOptions::new().read(true).open(path).await?;
        let pipe_client = Self {
            write_pipe: Arc::new(Mutex::new(write_pipe)),
            read_pipe: Arc::new(Mutex::new(read_pipe)),
            channels: Arc::new(RwLock::new(SerialPubChannels::new())),
        };
        info!("Pipe opened");
        Ok(pipe_client)
    }
}

#[async_trait]
impl NotificationHub for PipeClient {
    /// Send a message through channel
    async fn send(&self, data: HubMessage) -> Result<(), std::io::Error> {
        let mut write_pipe = self.write_pipe.lock().await;
        let raw_bytes = data.to_bytes()?;
        write_pipe.write_all(&raw_bytes).await?;
        Ok(())
    }

    /// List available topic channels
    async fn list_channels(&self) -> Result<Vec<HubChannelName>, std::io::Error> {
        let serial_channels = self.channels.read().await;
        Ok(serial_channels.iter().map(HubChannelName::from).collect())
    }

    /// Start client
    async fn start(&self, sender: broadcast::Sender<HubMessage>) -> Result<(), std::io::Error> {
        let read_pipe = Arc::clone(&self.read_pipe);
        let channels = Arc::clone(&self.channels);
        info!("Starting pipe...");

        tokio::spawn(async move {
            let read_pipe = Arc::clone(&read_pipe);
            loop {
                let mut read_pipe_lock = read_pipe.lock().await;
                let reader = BufReader::new(&mut *read_pipe_lock);
                let mut lines = reader.lines();
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if line.starts_with("##") {
                            let raw_serial_message = SerialRawMessage::from_str(line.as_str());
                            // serial client learns available channels by inspecting received data
                            match HubMessage::try_from(raw_serial_message) {
                                Ok(message) => {
                                    let mut serial_channels = channels.write().await;
                                    serial_channels
                                        .add(SerialChannelName::from(message.channel.clone()));
                                    if let Err(e) = sender.send(message) {
                                        error!("Serial port send error {:?}", e);
                                    }
                                }
                                Err(e) => error!("Pipe receive error {:?}", e),
                            }
                        } else {
                            warn!("Invalid pipe data. Waiting for valid channel prefix");
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        error!("Pipe error {:?}", e);
                        break;
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        Ok(())
    }
}
