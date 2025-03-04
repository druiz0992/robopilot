use serde::Serialize;
use serde_json::Value;

use crate::ports::sensor::SensorType;

#[derive(Debug, Serialize)]
pub struct SensorMessage {
    pub id: String,
    #[serde(flatten)]
    pub sensor_type: Box<dyn SensorType>,
    pub value: Value,
}
