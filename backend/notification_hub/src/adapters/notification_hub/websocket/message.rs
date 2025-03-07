use serde::{Deserialize, Serialize};

use crate::models::hub::{HubChannelName, HubData, HubMessage};

#[derive(Serialize, Debug, Clone, Deserialize)]
pub(crate) enum WsMessage {
    Subscribe(HubChannelName),
    Unsubscribe(HubChannelName),
    ListChannelsReq,
    ListChannelsResponse(Vec<HubChannelName>),
    Data(HubChannelName, HubData),
}

impl WsMessage {
    #[allow(dead_code)]
    pub fn subscribe(channel: &str) -> Result<Self, String> {
        let channel_name = HubChannelName::try_from(channel)?;
        Ok(WsMessage::Subscribe(channel_name))
    }

    pub fn subscribe_channel(channel: HubChannelName) -> Self {
        WsMessage::Subscribe(channel)
    }

    #[allow(dead_code)]
    pub fn unsubscribe(channel: &str) -> Result<Self, String> {
        let channel_name = HubChannelName::try_from(channel)?;
        Ok(WsMessage::Unsubscribe(channel_name))
    }

    pub fn unsubscribe_channel(channel: HubChannelName) -> Self {
        WsMessage::Unsubscribe(channel)
    }

    pub fn list_channels_req() -> Self {
        WsMessage::ListChannelsReq
    }

    #[allow(dead_code)]
    pub fn send_data(channel: &str, data: &str) -> Result<Self, String> {
        let channel_name = HubChannelName::try_from(channel)?;
        let data = data.parse::<HubData>().unwrap();
        Ok(WsMessage::Data(channel_name, data))
    }

    #[allow(dead_code)]
    pub fn send_data_channel(channel: HubChannelName, data: HubData) -> Self {
        WsMessage::Data(channel, data)
    }

    pub fn to_string(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

impl TryFrom<String> for WsMessage {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str::<WsMessage>(&value).map_err(|e| e.to_string())
    }
}
impl TryFrom<WsMessage> for HubMessage {
    type Error = String;

    fn try_from(value: WsMessage) -> Result<Self, Self::Error> {
        match value {
            WsMessage::Data(channel, data) => Ok(HubMessage::new(channel, data)),
            _ => Err("Invalid message type".to_string()),
        }
    }
}

impl From<HubMessage> for WsMessage {
    fn from(value: HubMessage) -> Self {
        WsMessage::Data(value.channel, value.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe() {
        let channel = "test_channel";
        let result = WsMessage::subscribe(channel);
        assert!(result.is_ok());
        if let Ok(WsMessage::Subscribe(channel_name)) = result {
            assert_eq!(channel_name.as_str(), channel);
        } else {
            panic!("Expected WsMessage::Subscribe");
        }
    }

    #[test]
    fn test_subscribe_channel() {
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let message = WsMessage::subscribe_channel(channel_name.clone());
        if let WsMessage::Subscribe(ch) = message {
            assert_eq!(ch, channel_name);
        } else {
            panic!("Expected WsMessage::Subscribe");
        }
    }

    #[test]
    fn test_unsubscribe() {
        let channel = "test_channel";
        let result = WsMessage::unsubscribe(channel);
        assert!(result.is_ok());
        if let Ok(WsMessage::Unsubscribe(channel_name)) = result {
            assert_eq!(channel_name.as_str(), channel);
        } else {
            panic!("Expected WsMessage::Unsubscribe");
        }
    }

    #[test]
    fn test_unsubscribe_channel() {
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let message = WsMessage::unsubscribe_channel(channel_name.clone());
        if let WsMessage::Unsubscribe(ch) = message {
            assert_eq!(ch, channel_name);
        } else {
            panic!("Expected WsMessage::Unsubscribe");
        }
    }

    #[test]
    fn test_list_channels() {
        let message = WsMessage::list_channels_req();
        if let WsMessage::ListChannelsReq = message {
            // Test passed
        } else {
            panic!("Expected WsMessage::ListChannels");
        }
    }

    #[test]
    fn test_send_data() {
        let channel = "test_channel";
        let data = "test_data";
        let result = WsMessage::send_data(channel, data);
        assert!(result.is_ok());
        if let Ok(WsMessage::Data(channel_name, _)) = result {
            assert_eq!(channel_name.as_str(), channel);
        } else {
            panic!("Expected WsMessage::Data");
        }
    }

    #[test]
    fn test_send_data_channel() {
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let data = "test_data".parse::<HubData>().unwrap();
        let message = WsMessage::send_data_channel(channel_name.clone(), data.clone());
        if let WsMessage::Data(ch, _) = message {
            assert_eq!(ch, channel_name);
        } else {
            panic!("Expected WsMessage::Data");
        }
    }
    #[test]
    fn test_subscribe_to_string() {
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let message = WsMessage::subscribe_channel(channel_name.clone());
        let result = message.to_string();
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let parsed_message: WsMessage = serde_json::from_str(&json_str).unwrap();
        if let WsMessage::Subscribe(ch) = parsed_message {
            assert_eq!(ch, channel_name);
        } else {
            panic!("Expected WsMessage::Subscribe");
        }
    }

    #[test]
    fn test_try_subscribe_from_string() {
        let json_str = r#"{"Subscribe":"test_channel"}"#.to_string();
        let result = WsMessage::try_from(json_str);
        assert!(result.is_ok());
        if let Ok(WsMessage::Subscribe(channel_name)) = result {
            assert_eq!(channel_name.as_str(), "test_channel");
        } else {
            panic!("Expected WsMessage::Subscribe");
        }
    }

    #[test]
    fn test_data_to_string() {
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let data = "test_data".parse::<HubData>().unwrap();
        let message = WsMessage::Data(channel_name.clone(), data.clone());
        let result = message.to_string();
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let parsed_message: WsMessage = serde_json::from_str(&json_str).unwrap();
        if let WsMessage::Data(ch, d) = parsed_message {
            assert_eq!(ch, channel_name);
            assert_eq!(d, data);
        } else {
            panic!("Expected WsMessage::Data");
        }
    }

    #[test]
    fn test_try_data_from_string() {
        let json_str = r#"{"Data":["test_channel","test_data1, test_data2"]}"#.to_string();
        let result = WsMessage::try_from(json_str);
        assert!(result.is_ok());
        if let Ok(WsMessage::Data(channel_name, data)) = result {
            assert_eq!(channel_name.as_str(), "test_channel");
            assert_eq!(data.as_str(), "test_data1, test_data2");
        } else {
            panic!("Expected WsMessage::Data");
        }
    }

    #[test]
    fn test_try_from_ws_message() {
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let data = "test_data".parse::<HubData>().unwrap();
        let ws_message = WsMessage::Data(channel_name.clone(), data.clone());
        let result = HubMessage::try_from(ws_message);
        assert!(result.is_ok());
        let hub_message = result.unwrap();
        assert_eq!(hub_message.channel, channel_name);
        assert_eq!(hub_message.data, data);
    }

    #[test]
    fn test_from_hub_message() {
        let channel_name = HubChannelName::try_from("test_channel").unwrap();
        let data = "test_data".parse::<HubData>().unwrap();
        let hub_message = HubMessage::new(channel_name.clone(), data.clone());
        let ws_message: WsMessage = hub_message.into();
        if let WsMessage::Data(ch, d) = ws_message {
            assert_eq!(ch, channel_name);
            assert_eq!(d, data);
        } else {
            panic!("Expected WsMessage::Data");
        }
    }
}
