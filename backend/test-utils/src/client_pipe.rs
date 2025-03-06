use async_trait::async_trait;
use log::{error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{broadcast, Mutex, RwLock};

use notification_hub::adapters::serial::channels::{SerialChannelName, SerialPubChannels};
use notification_hub::adapters::serial::message::SerialRawMessage;
use notification_hub::models::hub::{HubChannelName, HubMessage};
use notification_hub::ports::NotificationHub;

/// The `ClientPipe` struct represents a client that communicates via files, mimicking a serial port,  that can subscribe
/// to specific topic channels. It allows to send and receive messages on specific topics.

#[derive(Debug)]
pub struct PipeClient {
    write_pipe: Arc<Mutex<Option<File>>>,
    read_pipes: Option<Vec<Arc<Mutex<File>>>>,
    channels: Arc<RwLock<SerialPubChannels>>,
}

impl PipeClient {
    pub async fn new(
        write_path: Option<&str>,
        read_path: Option<Vec<&str>>,
    ) -> Result<Self, std::io::Error> {
        info!("Opening bidirectional pipe...");
        let mut write_pipe = None;
        if let Some(write_path) = write_path {
            write_pipe = Some(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(write_path)
                    .await?,
            );
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        let mut read_pipes = None;
        if let Some(read_paths) = read_path {
            let mut pipe_vec = Vec::new();
            for read_path in read_paths {
                pipe_vec.push(Arc::new(Mutex::new(
                    OpenOptions::new().read(true).open(read_path).await?,
                )));
            }
            read_pipes = Some(pipe_vec);
        }

        let pipe_client = Self {
            write_pipe: Arc::new(Mutex::new(write_pipe)),
            read_pipes: read_pipes,
            channels: Arc::new(RwLock::new(SerialPubChannels::new())),
        };
        info!("Pipe opened");
        Ok(pipe_client)
    }
    /// Start client
    async fn start_read_pipe(
        &self,
        sender: Option<broadcast::Sender<HubMessage>>,
        read_pipe: Arc<Mutex<File>>,
    ) -> Result<(), std::io::Error> {
        if let Some(sender) = sender {
            let read_pipe = Arc::clone(&read_pipe);
            let channels = Arc::clone(&self.channels);
            info!("Starting pipe...");

            tokio::spawn(async move {
                let read_pipe = Arc::clone(&read_pipe);
                loop {
                    let line = {
                        let mut read_pipe_lock = read_pipe.lock().await;
                        let reader = BufReader::new(&mut *read_pipe_lock);
                        reader.lines().next_line().await // Read one line
                    };
                    match line {
                        Ok(Some(line)) => {
                            let line = line
                                .strip_prefix('"')
                                .and_then(|s| s.strip_suffix('"'))
                                .unwrap_or(&line)
                                .to_string();
                            if line.starts_with("##") {
                                let raw_serial_message = SerialRawMessage::from_str(line.as_str());
                                // serial client learns available channels by inspecting received data
                                match HubMessage::try_from(raw_serial_message) {
                                    Ok(message) => {
                                        let mut serial_channels = channels.write().await;
                                        serial_channels
                                            .add(SerialChannelName::from(message.channel.clone()));
                                        info!(
                                            "New message from channel {:?} received by Pipe client",
                                            message.channel.clone()
                                        );
                                        if let Err(e) = sender.send(message) {
                                            error!("Pipe send error {:?}", e);
                                        }
                                    }
                                    Err(e) => error!("Pipe receive error {:?}", e),
                                }
                            } else {
                                warn!(
                                    "Invalid pipe data. Waiting for valid channel prefix: {:?}",
                                    line
                                );
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
        }
        Ok(())
    }
}

#[async_trait]
impl NotificationHub for PipeClient {
    /// Send a message through channel
    async fn send(&self, message: HubMessage) -> Result<(), std::io::Error> {
        let mut write_pipe = self.write_pipe.lock().await;
        let write_pipe = match &mut *write_pipe {
            Some(write_pipe) => write_pipe,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Write pipe is not available",
                ))
            }
        };
        let serial_message = SerialRawMessage::from(message);
        let raw_bytes = serial_message.to_bytes()?;
        info!("Send serial message: {:?}", serial_message);
        write_pipe.write_all(&raw_bytes).await?;
        Ok(())
    }

    /// List available topic channels
    async fn list_channels(&self) -> Result<Vec<HubChannelName>, std::io::Error> {
        let serial_channels = self.channels.read().await;
        Ok(serial_channels.iter().map(HubChannelName::from).collect())
    }

    /// Start client
    async fn start(
        &self,
        sender: Option<broadcast::Sender<HubMessage>>,
    ) -> Result<(), std::io::Error> {
        match &self.read_pipes {
            Some(read_pipes) => {
                for read_pipe in read_pipes {
                    let pipe = Arc::clone(read_pipe);
                    self.start_read_pipe(sender.clone(), pipe).await?;
                }
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Read pipe is not available",
                ))
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notification_hub::models::hub::HubData;

    #[tokio::test]
    async fn test_pipe_client_new() {
        let write_path = "/tmp/test_pipe";
        let client = PipeClient::new(Some(write_path), None).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_send_message() {
        let path = "/tmp/test_pipe_send";
        let client = PipeClient::new(Some(path), None).await.unwrap();
        let data = "1,2,3".parse::<HubData>().unwrap();
        let message = HubMessage::new(HubChannelName::try_from("test_channel").unwrap(), data);
        let result = client.send(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_channels() {
        let path = "/tmp/test_pipe_list";
        let client = PipeClient::new(Some(path), None).await.unwrap();
        let channels = client.list_channels().await.unwrap();
        assert!(channels.is_empty());
    }

    #[tokio::test]
    async fn test_start_pipe_client() {
        let path = "/tmp/test_pipe_start";
        let (sender2, mut receiver2) = broadcast::channel(16);

        let client_sender = PipeClient::new(Some(path), None).await.unwrap();
        let start_result = client_sender.start(None).await;
        assert!(start_result.is_ok());

        let client_receiver = PipeClient::new(None, Some(vec![path])).await.unwrap();
        let start_result = client_receiver.start(Some(sender2)).await;
        assert!(start_result.is_ok());

        // Simulate sending a message to the pipe
        let channel = HubChannelName::try_from("test_channel").unwrap();
        let data = "1,2,3".parse::<HubData>().unwrap();
        let message = HubMessage::new(channel, data);

        tokio::time::sleep(Duration::from_millis(200)).await;
        client_sender.send(message).await.unwrap();

        // Check if the message is received
        let message = receiver2.recv().await.unwrap();
        assert_eq!(message.channel.as_str(), "test_channel");
        assert_eq!(message.data.as_str(), "1,2,3");

        let channels = client_receiver.list_channels().await.unwrap();
        assert_eq!(
            channels,
            vec![HubChannelName::try_from("test_channel").unwrap()]
        );
    }
}
