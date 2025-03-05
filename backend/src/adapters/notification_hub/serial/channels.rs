use std::collections::HashSet;

use crate::models::hub::HubChannelName;

/// Module abstracts functionality for topic channels through a Serial port.

/// Functionality to convert from/to String to a Serial port topic channel. A serial port topic channel `TOPIC`
/// is a string with the format ##TOPIC##, where ## are separators in the serial port. `TOPIC` must follow same rules
/// as models::hub::hub_channel_name

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq)]
pub struct SerialChannelName(String);

impl SerialChannelName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn tag(&self) -> String {
        format!("##{}##", self.0)
    }
}

impl TryFrom<String> for SerialChannelName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SerialChannelName::try_from(value.as_str())
    }
}

impl TryFrom<&str> for SerialChannelName {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut name = value.to_ascii_lowercase();
        name = name
            .chars()
            .filter(|&c| !c.is_whitespace() && c != '\n' && c != '\r')
            .collect();

        if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(SerialChannelName(name));
        }
        Err("Invalid channel name. Only alphanumeric and '_' characters allowed.".to_string())
    }
}

impl From<SerialChannelName> for HubChannelName {
    fn from(value: SerialChannelName) -> Self {
        HubChannelName::try_from(value.as_str()).unwrap()
    }
}

impl From<HubChannelName> for SerialChannelName {
    fn from(value: HubChannelName) -> Self {
        SerialChannelName::try_from(value.as_str()).unwrap()
    }
}

/// Functionality to manage several topic channels
#[derive(Debug)]
pub struct SerialPubChannels {
    channels: HashSet<SerialChannelName>,
}

impl SerialPubChannels {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            channels: HashSet::new(),
        }
    }

    pub fn add(&mut self, channel_name: SerialChannelName) {
        self.channels.insert(channel_name);
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, channel_name: SerialChannelName) {
        self.channels.remove(&channel_name);
    }

    pub fn iter(&self) -> impl Iterator<Item = SerialChannelName> {
        self.channels.clone().into_iter()
    }
}

impl From<SerialChannelName> for String {
    fn from(value: SerialChannelName) -> Self {
        value.as_str().to_string()
    }
}

impl From<&SerialChannelName> for String {
    fn from(value: &SerialChannelName) -> Self {
        value.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_channel_name() {
        let mut channels = SerialPubChannels::new();
        let channel_name = SerialChannelName::try_from("example").unwrap();
        channels.add(channel_name.clone());
        assert!(channels.iter().any(|l| l == channel_name));
    }

    #[test]
    #[should_panic]
    fn test_add_channel_with_non_alphanumeric_chars() {
        SerialChannelName::try_from("example.com").unwrap();
    }

    #[test]
    fn test_add_channel_name_capital_letters() {
        let expected_channel_name = "example";
        let mut channels = SerialPubChannels::new();
        let channel_name = SerialChannelName::try_from("ExamPle").unwrap();
        channels.add(channel_name.clone());
        assert!(channels.iter().any(|l| l == channel_name));
        assert!(channel_name.as_str() == expected_channel_name);
    }

    #[test]
    fn test_add_channel_name_with_spaces() {
        let expected_channel_name = "example";
        let mut channels = SerialPubChannels::new();
        let channel_name = SerialChannelName::try_from(" exa mp l e   \n").unwrap();
        channels.add(channel_name.clone());
        assert!(channels.iter().any(|l| l == channel_name));
        assert!(channel_name.as_str() == expected_channel_name);
    }

    #[test]
    fn test_remove_channel_name() {
        let mut channels = SerialPubChannels::new();
        let channel_name = SerialChannelName::try_from("example").unwrap();
        channels.add(channel_name.clone());
        channels.remove(channel_name.clone());
        assert!(!channels.iter().any(|l| l == channel_name));
    }

    #[test]
    fn test_remove_nonexistent_channel_name() {
        let mut channels = SerialPubChannels::new();
        let channel_name1 = SerialChannelName::try_from("example").unwrap();
        let channel_name2 = SerialChannelName::try_from("example2").unwrap();
        channels.add(channel_name1.clone());
        channels.remove(channel_name2.clone());
    }

    #[test]
    fn test_iter_channels() {
        let mut channels = SerialPubChannels::new();
        let channel_name1 = SerialChannelName::try_from("example1").unwrap();
        let channel_name2 = SerialChannelName::try_from("example2").unwrap();
        channels.add(channel_name1.clone());
        channels.add(channel_name2.clone());
        let collected: Vec<_> = channels.iter().collect();
        assert!(collected.contains(&channel_name1));
        assert!(collected.contains(&channel_name2));
    }

    #[test]
    fn test_channel_name_from_str() {
        let channel_name_str = "example";
        let channel_name = SerialChannelName::try_from(channel_name_str).unwrap();
        assert_eq!(channel_name.0, channel_name_str.to_string());
    }

    #[test]
    fn test_channel_name_into_string() {
        let channel_name = SerialChannelName::try_from("example").unwrap();
        let channel_name_str: String = channel_name.into();
        assert_eq!(channel_name_str, "example".to_string());
    }

    #[test]
    fn test_channel_name_ref_into_string() {
        let channel_name = SerialChannelName::try_from("example").unwrap();
        let channel_name_str: String = (&channel_name).into();
        assert_eq!(channel_name_str, "example".to_string());
    }

    #[test]
    fn test_tag() {
        let channel_name = SerialChannelName::try_from("example").unwrap();
        assert_eq!(channel_name.tag(), "##example##");
    }
}
