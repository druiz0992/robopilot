use async_trait::async_trait;
use serde::{Serialize, Serializer};
use std::pin::Pin;
use tokio_stream::Stream;

use crate::models::SensorMessage;

pub trait SensorType: std::fmt::Debug + Send + Sync + 'static {
    fn sensor_type_as_str(&self) -> &'static str;
    fn clone_box(&self) -> Box<dyn SensorType>;
}

impl Serialize for dyn SensorType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.sensor_type_as_str())
    }
}

impl Clone for Box<dyn SensorType> {
    fn clone(&self) -> Box<dyn SensorType> {
        self.clone_box()
    }
}

#[async_trait]
pub trait Sensor: std::fmt::Debug + Send + Sync + 'static {
    fn get_type(&self) -> Box<dyn SensorType>;
    fn is_enabled(&self) -> bool;
    fn enable(&mut self);
    fn disable(&mut self);

    async fn get_data_stream(&mut self) -> Pin<Box<dyn Stream<Item = SensorMessage> + Send>>;
}
