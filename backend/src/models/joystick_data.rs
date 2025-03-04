use imu_common::traits::IMUUntimedSample;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JoystickData {
    id: String,
    value: JoystickValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JoystickValue {
    y: f32,
}

impl IMUUntimedSample for JoystickData {
    fn get_measurement(&self) -> Self {
        self.clone()
    }
}
