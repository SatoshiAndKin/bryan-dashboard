pub mod eip6963;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockHead {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
    pub base_fee: Option<u64>,
}

#[allow(dead_code)]
pub fn parse_hex_u64(s: &str) -> Option<u64> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).ok()
}

#[allow(dead_code)]
pub fn parse_block_head(val: &serde_json::Value) -> Option<BlockHead> {
    let obj = val.as_object()?;
    let number = parse_hex_u64(obj.get("number")?.as_str()?)?;
    let hash = obj
        .get("hash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let timestamp = obj
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(parse_hex_u64)
        .unwrap_or(0);
    let base_fee = obj
        .get("baseFeePerGas")
        .and_then(|v| v.as_str())
        .and_then(parse_hex_u64);
    Some(BlockHead {
        number,
        hash,
        timestamp,
        base_fee,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_u64() {
        assert_eq!(parse_hex_u64("0x0"), Some(0));
        assert_eq!(parse_hex_u64("0x1"), Some(1));
        assert_eq!(parse_hex_u64("0xff"), Some(255));
        assert_eq!(parse_hex_u64("0x13f0d40"), Some(20_909_376));
        assert_eq!(parse_hex_u64(""), None);
    }

    #[test]
    fn test_parse_block_head() {
        let json = serde_json::json!({
            "number": "0x13f0d40",
            "hash": "0xabc123",
            "timestamp": "0x65a1b2c3",
            "baseFeePerGas": "0x3b9aca00"
        });
        let bh = parse_block_head(&json).unwrap();
        assert_eq!(bh.number, 20_909_376);
        assert_eq!(bh.hash, "0xabc123");
        assert_eq!(bh.base_fee, Some(1_000_000_000));
    }
}
