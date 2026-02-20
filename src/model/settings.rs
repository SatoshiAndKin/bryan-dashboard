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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Legacy single RPC URL (kept for migration)
    #[serde(default)]
    pub rpc_url: String,
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
            rpc_url: String::new(),
            poll_interval_secs: 10,
            etherscan_api_key: String::new(),
            rpc_entries: Vec::new(),
            max_retries: 3,
            retry_backoff_ms: 1000,
        }
    }
}

#[allow(dead_code)]
impl AppSettings {
    pub fn is_websocket(&self) -> bool {
        // Check legacy URL first, then first rpc_entry
        if !self.rpc_url.is_empty() {
            let url = self.rpc_url.trim().to_lowercase();
            return url.starts_with("ws://") || url.starts_with("wss://");
        }
        self.rpc_entries
            .first()
            .map(|e| e.is_websocket())
            .unwrap_or(false)
    }

    pub fn is_http(&self) -> bool {
        let url = self.effective_rpc_url().unwrap_or_default().to_lowercase();
        url.starts_with("http://") || url.starts_with("https://")
    }

    pub fn has_rpc(&self) -> bool {
        self.effective_rpc_url().is_some()
    }

    /// Get the effective primary RPC URL (legacy or first entry)
    pub fn effective_rpc_url(&self) -> Option<String> {
        let legacy = self.rpc_url.trim();
        if !legacy.is_empty() {
            return Some(legacy.to_string());
        }
        self.rpc_entries.first().and_then(|e| e.primary_url())
    }

    /// Migrate legacy rpc_url to rpc_entries if needed
    pub fn migrate_legacy_rpc(&mut self) {
        if !self.rpc_url.is_empty() && self.rpc_entries.is_empty() {
            self.rpc_entries.push(RpcEntry {
                chain_id: 1,
                chain_name: "Ethereum Mainnet".to_string(),
                urls: self.rpc_url.clone(),
            });
        }
    }

    /// Get RPC entry for a specific chain ID
    pub fn rpc_for_chain(&self, chain_id: u64) -> Option<&RpcEntry> {
        self.rpc_entries.iter().find(|e| e.chain_id == chain_id)
    }

    /// Check if chain_id is unique before adding
    pub fn has_chain_id(&self, chain_id: u64) -> bool {
        self.rpc_entries.iter().any(|e| e.chain_id == chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_websocket() {
        let mut s = AppSettings::default();
        s.rpc_url = "wss://mainnet.infura.io/ws/v3/key".into();
        assert!(s.is_websocket());
        assert!(!s.is_http());
        assert!(s.has_rpc());
    }

    #[test]
    fn test_is_http() {
        let mut s = AppSettings::default();
        s.rpc_url = "https://mainnet.infura.io/v3/key".into();
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
    fn test_migrate_legacy_rpc() {
        let mut s = AppSettings::default();
        s.rpc_url = "https://mainnet.infura.io/v3/key".into();
        s.migrate_legacy_rpc();
        assert_eq!(s.rpc_entries.len(), 1);
        assert_eq!(s.rpc_entries[0].chain_id, 1);
        assert_eq!(s.rpc_entries[0].urls, s.rpc_url);
    }

    #[test]
    fn test_rpc_for_chain() {
        let mut s = AppSettings::default();
        s.rpc_entries.push(RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "https://eth.example.com".to_string(),
        });
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
        let mut s = AppSettings::default();
        s.rpc_entries.push(RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "https://eth.example.com".to_string(),
        });
        assert!(s.has_chain_id(1));
        assert!(!s.has_chain_id(137));
    }

    #[test]
    fn test_effective_rpc_url_legacy_first() {
        let mut s = AppSettings::default();
        s.rpc_url = "https://legacy.example.com".into();
        s.rpc_entries.push(RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "https://new.example.com".to_string(),
        });
        // Legacy URL takes precedence
        assert_eq!(
            s.effective_rpc_url(),
            Some("https://legacy.example.com".to_string())
        );
    }

    #[test]
    fn test_effective_rpc_url_from_entries() {
        let mut s = AppSettings::default();
        s.rpc_entries.push(RpcEntry {
            chain_id: 1,
            chain_name: "Ethereum".to_string(),
            urls: "https://new.example.com".to_string(),
        });
        assert_eq!(
            s.effective_rpc_url(),
            Some("https://new.example.com".to_string())
        );
    }
}
