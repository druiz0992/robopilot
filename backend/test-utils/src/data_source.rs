use imu_common::types::clock::Clock;
use log::{error, info};
use notification_hub::models::hub::{HubChannelName, HubData, HubMessage};
use notification_hub::ports::NotificationHub;
use rand::Rng;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};

pub struct DataSource<T: NotificationHub> {
    n_dims: usize,
    channel: HubChannelName,
    sender: broadcast::Sender<HubMessage>,
    receiver: broadcast::Receiver<HubMessage>,
    hub_client: T,
}

impl<T: NotificationHub> DataSource<T> {
    // Create a new DataSource with the number of dimensions
    pub fn new(client: T, n_dims: usize, channel: HubChannelName) -> Self {
        let (sender, receiver) = broadcast::channel(100);
        DataSource {
            n_dims,
            channel,
            sender,
            receiver,
            hub_client: client,
        }
    }

    // Start generating HubMessages every second
    pub async fn start(&mut self, delay_millis: u64) -> Result<(), String> {
        let _ = self.hub_client.start(None).await;
        tokio::time::sleep(Duration::from_millis(delay_millis)).await;
        generate_data_process(self.sender.clone(), self.channel.clone(), self.n_dims, 1000).await;

        while let Ok(message) = self.receiver.recv().await {
            info!("Received HubMessage: {:?}", message);
            if let Err(e) = self.hub_client.send(message).await {
                error!("Error sending message to hub client : {:?}", e);
            }
        }
        Ok(())
    }
}

async fn generate_data_process(
    sender: broadcast::Sender<HubMessage>,
    channel: HubChannelName,
    n_dims: usize,
    period_millis: u64,
) {
    let now = Clock::now().as_secs();
    let start = time::Instant::now();

    tokio::spawn(async move {
        loop {
            let timestamp = start.elapsed().as_secs_f64() + now;
            let data = generate_random_data(n_dims);
            let message = HubMessage {
                channel: channel.clone(),
                timestamp,
                data,
            };
            info!("New sample: {:?}", message);

            // Send the generated message to the sender
            if let Err(e) = sender.send(message) {
                error!("Error sending HubMessage: {:?}", e);
            }

            time::sleep(Duration::from_millis(period_millis)).await;
        }
    });
}

// Function to generate random f64 data for HubData
fn generate_random_data(n_dims: usize) -> HubData {
    let mut rng = rand::thread_rng();
    let values: Vec<f64> = (0..n_dims)
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
