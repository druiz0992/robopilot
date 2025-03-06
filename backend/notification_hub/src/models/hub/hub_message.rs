use imu_common::types::Clock;
use serde::{Deserialize, Serialize};

use super::{HubChannelName, HubData};

/// Represents a message in the hub system.
///
/// # Fields
///
/// * `channel` - The name of the channel the message is associated with.
/// * `timestamp` - The timestamp when the message was created.
/// * `data` - The data contained in the message.

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct HubMessage {
    pub channel: HubChannelName,
    pub timestamp: f64,
    pub data: HubData,
}

impl HubMessage {
    pub fn try_from_str(channel: &str, data: &str) -> Result<Self, String> {
        let channel = HubChannelName::try_from(channel)?;
        Ok(Self {
            channel,
            data: data.parse::<HubData>().unwrap(),
            timestamp: Clock::now().as_secs(),
        })
    }

    pub fn new(channel: HubChannelName, data: HubData) -> Self {
        Self {
            channel,
            data,
            timestamp: Clock::now().as_secs(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

impl TryFrom<Vec<u8>> for HubMessage {
    type Error = String;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value).map_err(|e| format!("Error converting to HubMessage {}", e))
    }
}

impl TryFrom<String> for HubMessage {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value).map_err(|e| format!("Error converting to HubMessage {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_channel_name_valid() {
        let valid_name = "valid_channel_name";
        let channel_name = HubChannelName::try_from(valid_name);
        assert!(channel_name.is_ok());
        assert_eq!(channel_name.unwrap().as_str(), valid_name);
    }

    #[test]
    fn test_hub_channel_name_invalid() {
        let invalid_name = "invalid channel name!";
        let channel_name = HubChannelName::try_from(invalid_name);
        assert!(channel_name.is_err());
    }

    #[test]
    fn test_hub_data_from_str() {
        let data = "  some data \n";
        let hub_data = data.parse::<HubData>().unwrap();
        assert_eq!(hub_data.as_str(), "some data");
    }

    #[test]
    fn test_hub_message_new_valid() {
        let channel = "valid_channel";
        let data = "some data";
        let message = HubMessage::try_from_str(channel, data);
        assert!(message.is_ok());
        let message = message.unwrap();
        assert_eq!(message.channel.as_str(), channel);
        assert_eq!(message.data.as_str(), "some data");
    }

    #[test]
    fn test_hub_message_new_invalid_channel() {
        let channel = "invalid channel!";
        let data = "some data";
        let message = HubMessage::try_from_str(channel, data);
        assert!(message.is_err());
    }

    #[test]
    fn test_hub_message_to_bytes() {
        let channel = "valid_channel";
        let data = "some data";
        let message = HubMessage::try_from_str(channel, data).unwrap();
        let bytes = message.to_bytes();
        assert!(bytes.is_ok());
    }

    #[test]
    fn test_hub_channel_name_to_string() {
        let valid_name = "valid_channel_name";
        let channel_name = HubChannelName::try_from(valid_name).unwrap();
        assert_eq!(channel_name.as_str(), valid_name);
    }

    #[test]
    fn test_hub_data_to_string() {
        let data = "some data";
        let hub_data = data.parse::<HubData>().unwrap();
        assert_eq!(hub_data.as_str(), data);
    }

    #[test]
    fn test_hub_message_try_from_string() {
        let channel = "valid_channel";
        let data = "some data";
        let message = HubMessage::try_from_str(channel, data).unwrap();
        let message_string = serde_json::to_string(&message).unwrap();
        let deserialized_message = HubMessage::try_from(message_string).unwrap();
        assert_eq!(message.channel, deserialized_message.channel);
        assert_eq!(message.data, deserialized_message.data);
    }

    #[test]
    fn test_hub_message_try_from_bytes() {
        let channel = "valid_channel";
        let data = "some data";
        let message = HubMessage::try_from_str(channel, data).unwrap();
        let message_bytes = message.to_bytes().unwrap();
        let deserialized_message = HubMessage::try_from(message_bytes).unwrap();
        assert_eq!(message.channel, deserialized_message.channel);
        assert_eq!(message.data, deserialized_message.data);
    }
}
