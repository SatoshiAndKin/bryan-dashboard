use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub rpc_url: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u32,
}

fn default_poll_interval() -> u32 {
    10
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            rpc_url: String::new(),
            poll_interval_secs: 10,
        }
    }
}

#[allow(dead_code)]
impl AppSettings {
    pub fn is_websocket(&self) -> bool {
        let url = self.rpc_url.trim().to_lowercase();
        url.starts_with("ws://") || url.starts_with("wss://")
    }

    pub fn is_http(&self) -> bool {
        let url = self.rpc_url.trim().to_lowercase();
        url.starts_with("http://") || url.starts_with("https://")
    }

    pub fn has_rpc(&self) -> bool {
        !self.rpc_url.trim().is_empty() && (self.is_websocket() || self.is_http())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_websocket() {
        let s = AppSettings {
            rpc_url: "wss://mainnet.infura.io/ws/v3/key".into(),
            ..Default::default()
        };
        assert!(s.is_websocket());
        assert!(!s.is_http());
        assert!(s.has_rpc());
    }

    #[test]
    fn test_is_http() {
        let s = AppSettings {
            rpc_url: "https://mainnet.infura.io/v3/key".into(),
            ..Default::default()
        };
        assert!(!s.is_websocket());
        assert!(s.is_http());
        assert!(s.has_rpc());
    }

    #[test]
    fn test_empty_rpc() {
        let s = AppSettings::default();
        assert!(!s.has_rpc());
    }

    #[test]
    fn test_default_poll_interval() {
        let s = AppSettings::default();
        assert_eq!(s.poll_interval_secs, 10);
    }
}
