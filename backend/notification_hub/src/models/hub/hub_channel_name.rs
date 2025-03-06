use serde::{Deserialize, Serialize};

/// Represents a channel name in the hub.
///
/// This struct ensures that the channel name adheres to specific rules:
/// - Only alphanumeric characters and underscores are allowed.
/// - No spaces are allowed in the middle of the string.
/// - Leading and trailing whitespaces, newlines, and carriage returns are trimmed.
/// - The channel name is converted to lowercase.
///

#[derive(Serialize, Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct HubChannelName(String);

impl HubChannelName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for HubChannelName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        HubChannelName::try_from(value.as_str())
    }
}

impl TryFrom<&str> for HubChannelName {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Trim leading and trailing spaces, newlines, and carriage returns
        let trimmed = value.trim_matches(|c: char| c.is_whitespace() || c == '\n' || c == '\r');

        // Check if there are any newlines or carriage returns in the middle of the string
        if trimmed.contains('\n') || trimmed.contains('\r') {
            return Err(
                "Invalid channel name: Newlines are not allowed in the middle.".to_string(),
            );
        }

        // Ensure only alphanumeric characters and '_' exist, and no spaces in the middle
        if trimmed
            .chars()
            .any(|c| !(c.is_alphanumeric() || c == '_' || c.is_whitespace()))
        {
            return Err(
                "Invalid channel name: Only alphanumeric characters and '_' are allowed."
                    .to_string(),
            );
        }

        // Reject spaces in the middle of the string
        if trimmed.contains(' ') {
            return Err(
                "Invalid channel name: Whitespace is only allowed at the beginning or end."
                    .to_string(),
            );
        }

        // Return the valid channel name (in lowercase)
        Ok(HubChannelName(trimmed.to_ascii_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_channel_name() {
        let valid_name = "valid_channel_123";
        let hub_channel_name = HubChannelName::try_from(valid_name);
        assert!(hub_channel_name.is_ok());
        assert_eq!(hub_channel_name.unwrap().as_str(), valid_name);
    }

    #[test]
    fn test_invalid_channel_name_with_spaces() {
        let invalid_name = "invalid channel";
        let hub_channel_name = HubChannelName::try_from(invalid_name);
        assert!(hub_channel_name.is_err());
    }

    #[test]
    fn test_invalid_channel_name_with_special_chars() {
        let invalid_name = "invalid@channel!";
        let hub_channel_name = HubChannelName::try_from(invalid_name);
        assert!(hub_channel_name.is_err());
    }

    #[test]
    fn test_channel_name_with_uppercase() {
        let name_with_uppercase = "Valid_Channel";
        let hub_channel_name = HubChannelName::try_from(name_with_uppercase);
        assert!(hub_channel_name.is_ok());
        assert_eq!(hub_channel_name.unwrap().as_str(), "valid_channel");
    }

    #[test]
    fn test_channel_name_with_newlines() {
        let name_with_newlines = "valid\nchannel";
        let hub_channel_name = HubChannelName::try_from(name_with_newlines);
        assert!(hub_channel_name.is_err());
    }

    #[test]
    fn test_channel_name_with_carriage_returns() {
        let name_with_carriage_returns = "valid\rchannel";
        let hub_channel_name = HubChannelName::try_from(name_with_carriage_returns);
        assert!(hub_channel_name.is_err());
    }

    #[test]
    fn test_channel_name_with_newlines_ok() {
        let name_with_newlines = "validchannel\n";
        let hub_channel_name = HubChannelName::try_from(name_with_newlines);
        assert!(hub_channel_name.is_ok());
        assert_eq!(hub_channel_name.unwrap().as_str(), "validchannel");
    }

    #[test]
    fn test_channel_name_with_carriage_returns_ok() {
        let name_with_carriage_returns = "validchannel\r";
        let hub_channel_name = HubChannelName::try_from(name_with_carriage_returns);
        assert!(hub_channel_name.is_ok());
        assert_eq!(hub_channel_name.unwrap().as_str(), "validchannel");
    }

    #[test]
    fn test_channel_name_to_string() {
        let valid_name = "valid_channel_123";
        let hub_channel_name = HubChannelName::try_from(valid_name).unwrap();
        assert_eq!(hub_channel_name.as_str(), valid_name.to_string());
    }

    #[test]
    fn test_try_from_string() {
        let valid_name = "valid_channel_123".to_string();
        let hub_channel_name = HubChannelName::try_from(valid_name.clone());
        assert!(hub_channel_name.is_ok());
        assert_eq!(hub_channel_name.unwrap().as_str(), valid_name);
    }
}
