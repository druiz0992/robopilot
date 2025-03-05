use imu_common::types::clock::Clock;
use log::{error, info};
use rand::Rng;
use robopilot::adapters::serial::PipeClient;
use robopilot::models::hub::{HubChannelName, HubData, HubMessage};
use robopilot::ports::NotificationHub;
use robopilot::services::hub::HubManager;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let path_pipe = "/tmp/serial_pipe";
    let mut hub = HubManager::new();
    if let Ok(pipe_client) = PipeClient::new(path_pipe).await {
        hub.add(Box::new(pipe_client));
    }
    hub.start().await?;

    start_pipe_data_sources(hub.get_sender(), path_pipe).await;

    let channels = hub.list_channels().await.unwrap();
    info!("ZZZZZZZZZZz Available channels: {:?}", channels);

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}

async fn start_pipe_data_sources(hub_sender: broadcast::Sender<HubMessage>, path_pipe: &str) {
    // generate two data sources
    let mut accelerometer = PipeDataSource::new(
        3,
        HubChannelName::try_from("Accelerometer").unwrap(),
        path_pipe,
    )
    .await;

    let mut orientation = PipeDataSource::new(
        4,
        HubChannelName::try_from("Orientation").unwrap(),
        path_pipe,
    )
    .await;

    tokio::spawn(async move {
        let accelerometer_sender = hub_sender.clone();
        let orientation_sender = hub_sender.clone();

        tokio::try_join!(
            accelerometer.start(accelerometer_sender),
            orientation.start(orientation_sender)
        )
        .unwrap();
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
}

struct PipeDataSource {
    receiver: broadcast::Receiver<HubMessage>,
    data_source: DataSource,
    pipe_client: PipeClient,
}

impl PipeDataSource {
    async fn new(n_dims: usize, channel: HubChannelName, pipe_path: &str) -> Self {
        let data_source = DataSource::new(n_dims, channel);
        let receiver = data_source.sender.subscribe();
        let pipe_client = PipeClient::new(pipe_path).await.unwrap();
        Self {
            data_source,
            receiver,
            pipe_client,
        }
    }

    async fn start(&mut self, sender: broadcast::Sender<HubMessage>) -> Result<(), String> {
        self.data_source.start().await;
        //self.pipe_client.start().await;
        while let Ok(message) = self.receiver.recv().await {
            info!("Received HubMessage: {:?}", message);
            sender.send(message).unwrap();
        }
        Ok(())
    }
}
struct DataSource {
    n_dims: usize,
    channel: HubChannelName,
    pub sender: broadcast::Sender<HubMessage>,
}

impl DataSource {
    // Create a new DataSource with the number of dimensions
    fn new(n_dims: usize, channel: HubChannelName) -> Self {
        let (sender, _) = broadcast::channel(100);
        DataSource {
            n_dims,
            channel,
            sender,
        }
    }

    // Function to generate random f64 data for HubData
    fn generate_random_data(&self) -> HubData {
        let mut rng = rand::thread_rng();
        let values: Vec<f64> = (0..self.n_dims)
            .map(|_| rng.gen_range(-100.0..100.0)) // Random f64 between -100 and 100
            .collect();

        // Join the values into a comma-separated string
        let data = values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");
        data.parse::<HubData>().unwrap()
    }

    // Start generating HubMessages every second
    async fn start(&self) {
        let now = Clock::now().as_secs();
        let start = time::Instant::now();

        loop {
            let timestamp = start.elapsed().as_secs_f64() + now;
            let data = self.generate_random_data();
            let message = HubMessage {
                channel: self.channel.clone(),
                timestamp,
                data,
            };
            info!("New sample: {:?}", message);

            // Send the generated message to the sender
            if let Err(e) = self.sender.send(message) {
                error!("Error sending HubMessage: {:?}", e);
            }

            time::sleep(Duration::from_secs(1)).await;
        }
    }
}
