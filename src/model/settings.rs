use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcEntry {
    pub chain_id: u64,
    pub chain_name: String,
    /// Comma-separated URLs for fallback. Multiple providers for same chain.
    pub urls: String,
}

impl RpcEntry {
    pub fn url_list(&self) -> Vec<String> {
        self.urls
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn primary_url(&self) -> Option<String> {
        self.url_list().into_iter().next()
    }

    pub fn is_websocket(&self) -> bool {
        self.primary_url()
            .map(|u| {
                let u = u.to_lowercase();
                u.starts_with("ws://") || u.starts_with("wss://")
            })
            .unwrap_or(false)
    }

    pub fn is_http(&self) -> bool {
        self.primary_url()
            .map(|u| {
                let u = u.to_lowercase();
                u.starts_with("http://") || u.starts_with("https://")
            })
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u32,
    #[serde(default)]
    pub etherscan_api_key: String,
    #[serde(default)]
    pub rpc_entries: Vec<RpcEntry>,
    #[serde(default)]
    pub max_retries: u32,
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,
}

fn default_poll_interval() -> u32 {
    10
}

fn default_retry_backoff_ms() -> u64 {
    1000
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            poll_interval_secs: 10,
            etherscan_api_key: String::new(),
            rpc_entries: vec![RpcEntry {
                chain_id: 1,
                chain_name: "Ethereum Mainnet".to_string(),
                urls: String::new(),
            }],
            max_retries: 3,
            retry_backoff_ms: 1000,
        }
    }
}

impl AppSettings {
    pub fn rpc_for_chain(&self, chain_id: u64) -> Option<&RpcEntry> {
        self.rpc_entries.iter().find(|e| e.chain_id == chain_id)
    }

    pub fn has_chain_id(&self, chain_id: u64) -> bool {
        self.rpc_entries.iter().any(|e| e.chain_id == chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_has_chain_1_entry() {
        let s = AppSettings::default();
        assert_eq!(s.rpc_entries.len(), 1);
        assert_eq!(s.rpc_entries[0].chain_id, 1);
        assert_eq!(s.rpc_entries[0].chain_name, "Ethereum Mainnet");
        assert!(s.rpc_entries[0].urls.is_empty());
    }

    #[test]
    fn test_rpc_entry_is_websocket() {
        let entry = RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "wss://mainnet.infura.io/ws/v3/key".to_string(),
        };
        assert!(entry.is_websocket());
        assert!(!entry.is_http());
    }

    #[test]
    fn test_rpc_entry_is_http() {
        let entry = RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "https://mainnet.infura.io/v3/key".to_string(),
        };
        assert!(!entry.is_websocket());
        assert!(entry.is_http());
    }

    #[test]
    fn test_default_poll_interval() {
        let s = AppSettings::default();
        assert_eq!(s.poll_interval_secs, 10);
    }

    #[test]
    fn test_rpc_entry_url_list() {
        let entry = RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "https://rpc1.example.com, https://rpc2.example.com".to_string(),
        };
        let urls = entry.url_list();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://rpc1.example.com");
        assert_eq!(urls[1], "https://rpc2.example.com");
    }

    #[test]
    fn test_rpc_entry_primary_url() {
        let entry = RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "https://rpc1.example.com, https://rpc2.example.com".to_string(),
        };
        assert_eq!(
            entry.primary_url(),
            Some("https://rpc1.example.com".to_string())
        );
    }

    #[test]
    fn test_rpc_for_chain() {
        let mut s = AppSettings::default();
        s.rpc_entries[0].urls = "https://eth.example.com".into();
        s.rpc_entries.push(RpcEntry {
            chain_id: 137,
            chain_name: "Polygon".to_string(),
            urls: "https://polygon.example.com".to_string(),
        });
        assert!(s.rpc_for_chain(1).is_some());
        assert!(s.rpc_for_chain(137).is_some());
        assert!(s.rpc_for_chain(42).is_none());
    }

    #[test]
    fn test_has_chain_id() {
        let s = AppSettings::default();
        assert!(s.has_chain_id(1));
        assert!(!s.has_chain_id(137));
    }
}
