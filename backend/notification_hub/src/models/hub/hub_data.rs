use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The `HubData` struct represents a wrapper around a `String` that provides
/// additional functionality for handling and manipulating string data.

#[derive(Serialize, Debug, Clone, Deserialize, PartialEq)]
pub struct HubData(String);

impl HubData {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for HubData {
    type Err = String;
    fn from_str(data: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            data.trim_matches(|c| c == ' ' || c == '\n' || c == '\r')
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let data = "  example data  ".parse::<HubData>().unwrap();
        assert_eq!(data, HubData("example data".to_string()));
    }

    #[test]
    fn test_as_str() {
        let data = "example data".parse::<HubData>().unwrap();
        assert_eq!(data.as_str(), "example data");
    }

    #[test]
    fn test_trim_whitespace() {
        let data = "  example data  \n\r".parse::<HubData>().unwrap();
        assert_eq!(data, HubData("example data".to_string()));
    }

    #[test]
    fn test_empty_string() {
        let data = "   ".parse::<HubData>().unwrap();
        assert_eq!(data, HubData("".to_string()));
    }
}
