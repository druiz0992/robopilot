use serde::{Deserialize, Serialize};

/// The `HubData` struct represents a wrapper around a `String` that provides
/// additional functionality for handling and manipulating string data.

#[derive(Serialize, Debug, Clone, Deserialize, PartialEq)]
pub struct HubData(String);

impl HubData {
    pub fn from_str(data: &str) -> Self {
        Self(
            data.trim_matches(|c| c == ' ' || c == '\n' || c == '\r')
                .to_string(),
        )
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let data = HubData::from_str("  example data  ");
        assert_eq!(data, HubData("example data".to_string()));
    }

    #[test]
    fn test_as_str() {
        let data = HubData::from_str("example data");
        assert_eq!(data.as_str(), "example data");
    }

    #[test]
    fn test_to_string() {
        let data = HubData::from_str("example data");
        assert_eq!(data.to_string(), "example data".to_string());
    }

    #[test]
    fn test_trim_whitespace() {
        let data = HubData::from_str("  example data  \n\r");
        assert_eq!(data, HubData("example data".to_string()));
    }

    #[test]
    fn test_empty_string() {
        let data = HubData::from_str("   ");
        assert_eq!(data, HubData("".to_string()));
    }
}
