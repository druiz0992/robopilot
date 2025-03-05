use async_trait::async_trait;
use log::{error, info, warn};
use serialport::SerialPort;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::{broadcast, RwLock};
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
pub struct SerialClient {
    port: Arc<RwLock<SerialStream>>,
    serial_channels: Arc<RwLock<SerialPubChannels>>,
}

impl SerialClient {
    pub fn new(port: &str, baud_rate: u32) -> Result<Self, std::io::Error> {
        info!("Opening serial port {} with params {}...", port, baud_rate);
        let mut port = tokio_serial::new(port, baud_rate)
            .open_native_async()
            .inspect_err(|_| {
                error!("Serial port at {} not ready", port);
            })?;
        port.set_parity(Parity::None)?;
        port.set_stop_bits(StopBits::One)?;
        port.set_data_bits(DataBits::Eight)?;
        let handler = Self {
            port: Arc::new(RwLock::new(port)),
            serial_channels: Arc::new(RwLock::new(SerialPubChannels::new())),
        };
        info!("Serial port opened...");
        Ok(handler)
    }
}

#[async_trait]
impl NotificationHub for SerialClient {
    /// Send a message through channel
    async fn send(&self, data: HubMessage) -> Result<(), std::io::Error> {
        let raw_bytes = data.to_bytes()?;
        let mut port = self.port.write().await;
        tokio::io::AsyncWriteExt::write_all(&mut *port, &raw_bytes).await
    }

    /// List available topic channels
    async fn list_channels(&self) -> Result<Vec<HubChannelName>, std::io::Error> {
        let serial_channels = self.serial_channels.read().await;
        Ok(serial_channels.iter().map(HubChannelName::from).collect())
    }

    /// Start client
    async fn start(&self, sender: broadcast::Sender<HubMessage>) -> Result<(), std::io::Error> {
        let port = self.port.clone();
        let mut line_buffer = Vec::new();
        let serial_channels = Arc::clone(&self.serial_channels);
        info!("Starting Serial port...");

        tokio::spawn(async move {
            let mut buffer = vec![0u8; BUFFER_SIZE];
            loop {
                let mut port_write = port.write().await;
                match port_write.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        line_buffer.extend_from_slice(&buffer[..n]);
                        while let Some(pos) = line_buffer.iter().position(|&b| b == b'\n') {
                            let line = String::from_utf8_lossy(&line_buffer[..=pos]).to_string();

                            if line.starts_with("##") {
                                let raw_serial_message = SerialRawMessage::from_str(line.as_str());
                                // serial client learns available channels by inspecting received data
                                match HubMessage::try_from(raw_serial_message) {
                                    Ok(message) => {
                                        let mut serial_channels = serial_channels.write().await;
                                        serial_channels
                                            .add(SerialChannelName::from(message.channel.clone()));
                                        if let Err(e) = sender.send(message) {
                                            error!("Serial port send error {:?}", e);
                                        }
                                    }
                                    Err(e) => error!("Serial port receive error {:?}", e),
                                }
                            } else {
                                warn!("Invalid serial data. Waiting for valid channel prefix");
                            }
                            line_buffer.drain(0..=pos);
                        }
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        error!("Serial port error {:?}", e);
                        break;
                    }
                }
                drop(port_write);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORT: &str = "/dev/ttyACM0";
    const BAUD_RATE: u32 = 9600;

    #[tokio::test]
    #[ignore]
    async fn test_serial() {
        let client = SerialClient::new(PORT, BAUD_RATE).unwrap();
        let (sender, mut receiver) = broadcast::channel(100);
        client.start(sender).await.unwrap();

        let message = HubMessage::try_from_str("test_channel", "test_data").unwrap();
        let result = client.send(message).await;
        assert!(result.is_ok());

        if let Ok(response) = receiver.recv().await {
            assert_eq!(
                response.channel,
                HubChannelName::try_from("acceleration").unwrap()
            );
        }

        let channels = client.list_channels().await.unwrap();
        assert_eq!(
            channels,
            vec![HubChannelName::try_from("acceleration").unwrap()]
        );

        let result = client
            .subscribe(HubChannelName::try_from("test_channel").unwrap())
            .await;
        assert!(result.is_ok());

        let result = client
            .unsubscribe(HubChannelName::try_from("test_channel").unwrap())
            .await;
        assert!(result.is_ok());
    }
}
