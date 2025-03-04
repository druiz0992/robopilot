use crate::{
    models::SensorMessage,
    ports::{Sensor, SensorType},
};
use async_trait::async_trait;
use serde_json::json;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use uuid::Uuid;

// Define the different sensor types
#[derive(Debug, Clone)]
pub enum MockSensorType {
    Orientation,
    WheelOdometry,
}

// Implement `SensorType` for the enum
impl SensorType for MockSensorType {
    fn sensor_type_as_str(&self) -> &'static str {
        match self {
            MockSensorType::Orientation => "Orientation",
            MockSensorType::WheelOdometry => "WheelOdometry",
        }
    }

    fn clone_box(&self) -> Box<dyn SensorType> {
        Box::new(self.clone()) // Enum is `Clone`, so we can return a boxed clone
    }
}

// Define a single sensor struct that can represent both sensor types
#[derive(Debug)]
pub struct MockSensor {
    id: Uuid,
    enabled: bool,
    sensor_type: MockSensorType,
}

impl MockSensor {
    pub fn new(sensor_type: MockSensorType) -> Box<Self> {
        Box::new(Self {
            id: Uuid::new_v4(),
            enabled: false,
            sensor_type,
        })
    }
}

#[async_trait]
impl Sensor for MockSensor {
    fn get_type(&self) -> Box<dyn SensorType> {
        Box::new(self.sensor_type.clone())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }

    async fn get_data_stream(&mut self) -> Pin<Box<dyn Stream<Item = SensorMessage> + Send>> {
        let (tx, rx) = mpsc::channel(10);
        let id = self.id.clone();
        let sensor_type = self.sensor_type.clone();

        tokio::spawn(async move {
            loop {
                let value: f32 = rand::random::<f32>() * 100.0;
                let message = SensorMessage {
                    id: id.to_string(),
                    sensor_type: Box::new(sensor_type.clone()),
                    value: json!(value),
                };

                if tx.send(message).await.is_err() {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
    }
}
