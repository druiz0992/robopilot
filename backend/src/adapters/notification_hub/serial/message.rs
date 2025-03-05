use imu_common::types::Clock;

use super::channels::SerialChannelName;

use crate::models::hub::{HubChannelName, HubData, HubMessage};
use std::convert::TryFrom;

struct SerialData(String);

impl SerialData {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn from_str(data: &str) -> Self {
        SerialData(data.to_string())
    }
}

#[derive(Debug)]
pub struct SerialRawMessage(String);

impl SerialRawMessage {
    pub fn from_str(data: &str) -> Self {
        Self(data.to_string())
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl SerialRawMessage {
    fn extract_info(&self) -> Option<(SerialChannelName, SerialData)> {
        let raw_data = self.0.as_str();
        if let Some(start) = raw_data.find("##") {
            if let Some(end) = raw_data[start + 2..].find("##") {
                let raw_channel_name = &raw_data[start + 2..start + 2 + end];
                let data = raw_data[start + 2 + end + 2..]
                    .trim_matches(|c| c == '\n' || c == '\r' || c == ' ');
                if let Ok(channel_name) = SerialChannelName::try_from(raw_channel_name) {
                    return Some((channel_name, SerialData(data.to_string())));
                }
            }
        }
        None
    }
}

impl From<(SerialChannelName, SerialData)> for HubMessage {
    fn from(value: (SerialChannelName, SerialData)) -> Self {
        HubMessage {
            timestamp: Clock::now().as_secs(),
            channel: HubChannelName::from(value.0),
            data: HubData::from(value.1),
        }
    }
}
impl TryFrom<SerialRawMessage> for HubMessage {
    type Error = String;

    fn try_from(value: SerialRawMessage) -> Result<Self, Self::Error> {
        if let Some((channel_name, serial_data)) = value.extract_info() {
            return Ok(HubMessage::new(
                HubChannelName::from(channel_name),
                HubData::from(serial_data),
            ));
        }
        Err("Couldn't convert serial raw message to serial message.".to_string())
    }
}

impl From<SerialData> for HubData {
    fn from(value: SerialData) -> Self {
        value.as_str().parse::<HubData>().unwrap()
    }
}

impl From<HubData> for SerialData {
    fn from(value: HubData) -> Self {
        SerialData::from_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_data_from_str() {
        let data = "test_data";
        let serial_data = SerialData::from_str(data);
        assert_eq!(serial_data.as_str(), data);
    }

    #[test]
    fn test_serial_raw_message_from_str() {
        let data = "##channel##data";
        let serial_raw_message = SerialRawMessage::from_str(data);
        assert_eq!(serial_raw_message.as_str(), data);
    }

    #[test]
    fn test_serial_raw_message_extract_info() {
        let data = "##channel##data";
        let serial_raw_message = SerialRawMessage::from_str(data);
        let extracted_info = serial_raw_message.extract_info();
        assert!(extracted_info.is_some());
        let (channel_name, serial_data) = extracted_info.unwrap();
        assert_eq!(
            channel_name,
            SerialChannelName::try_from("channel").unwrap()
        );
        assert_eq!(serial_data.as_str(), "data");
    }

    #[test]
    fn test_serial_raw_message_extract_info_invalid() {
        let data = "invalid_data";
        let serial_raw_message = SerialRawMessage::from_str(data);
        let extracted_info = serial_raw_message.extract_info();
        assert!(extracted_info.is_none());
    }

    #[test]
    fn test_hub_message_from_serial_raw_message() {
        let data = "##channel##data";
        let serial_raw_message = SerialRawMessage::from_str(data);
        let hub_message = HubMessage::try_from(serial_raw_message);
        assert!(hub_message.is_ok());
        let hub_message = hub_message.unwrap();
        assert_eq!(
            hub_message.channel,
            HubChannelName::from(SerialChannelName::try_from("channel").unwrap())
        );
        assert_eq!(hub_message.data.as_str(), "data");
    }

    #[test]
    fn test_hub_message_from_serial_raw_message_invalid() {
        let data = "invalid_data";
        let serial_raw_message = SerialRawMessage::from_str(data);
        let hub_message = HubMessage::try_from(serial_raw_message);
        assert!(hub_message.is_err());
    }

    #[test]
    fn test_serial_data_to_hub_data() {
        let serial_data = SerialData::from_str("data");
        let hub_data: HubData = serial_data.into();
        assert_eq!(hub_data.as_str(), "data");
    }

    #[test]
    fn test_hub_data_to_serial_data() {
        let hub_data = "data".parse::<HubData>().unwrap();
        let serial_data: SerialData = hub_data.into();
        assert_eq!(serial_data.as_str(), "data");
    }
}
