use serde::{Deserialize, Serialize};

/// EIP-6963 provider info (from eip6963:announceProvider events)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletProviderInfo {
    pub uuid: String,
    pub name: String,
    pub icon: String,
    pub rdns: String,
}

/// Discovered EIP-6963 provider: info + the JS provider object reference index
#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredProvider {
    pub info: WalletProviderInfo,
    /// Index into the JS-side provider array (used to call back into JS)
    pub index: usize,
}

/// Request JSON-RPC via an EIP-6963 discovered provider.
/// `provider_index` is the index returned in DiscoveredProvider.
/// This sends an `eth_*` request through the provider's `request` method.
#[cfg(target_arch = "wasm32")]
pub async fn eip6963_request(
    provider_index: usize,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let js_code = format!(
        r#"
        (async function() {{
            const providers = window.__eip6963_providers || [];
            if ({idx} >= providers.length) throw new Error('Provider not found');
            const provider = providers[{idx}].provider;
            const result = await provider.request({{
                method: '{method}',
                params: {params}
            }});
            return JSON.stringify(result);
        }})()
        "#,
        idx = provider_index,
        method = method,
        params = params.to_string(),
    );

    let promise = js_sys::eval(&js_code).map_err(|e| format!("{:?}", e))?;
    let promise = js_sys::Promise::from(promise);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("{:?}", e))?;
    let text = result.as_string().ok_or("Expected string result")?;
    serde_json::from_str(&text).map_err(|e| format!("{e}"))
}

/// Discover available EIP-6963 wallet providers by dispatching the request event
/// and collecting announced providers via a JS snippet.
/// Stores discovered providers in `window.__eip6963_providers` for later use.
#[cfg(target_arch = "wasm32")]
pub fn discover_providers() -> Vec<WalletProviderInfo> {
    use wasm_bindgen::prelude::*;

    let js_code = r#"
    (function() {
        window.__eip6963_providers = window.__eip6963_providers || [];
        // Only set up listener once
        if (!window.__eip6963_listener_set) {
            window.__eip6963_listener_set = true;
            window.addEventListener('eip6963:announceProvider', function(event) {
                const info = event.detail.info;
                const provider = event.detail.provider;
                // Deduplicate by uuid
                const existing = window.__eip6963_providers.findIndex(p => p.info.uuid === info.uuid);
                if (existing === -1) {
                    window.__eip6963_providers.push({ info: info, provider: provider });
                }
            });
        }
        // Request providers to announce themselves
        window.dispatchEvent(new Event('eip6963:requestProvider'));
        // Return current list
        return JSON.stringify(window.__eip6963_providers.map((p, i) => ({
            uuid: p.info.uuid || '',
            name: p.info.name || '',
            icon: p.info.icon || '',
            rdns: p.info.rdns || '',
            index: i
        })));
    })()
    "#;

    let result = match js_sys::eval(js_code) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let json_str = match result.as_string() {
        Some(s) => s,
        None => return Vec::new(),
    };

    #[derive(Deserialize)]
    struct RawProvider {
        uuid: String,
        name: String,
        icon: String,
        rdns: String,
    }

    let raw: Vec<RawProvider> = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    raw.into_iter()
        .map(|p| WalletProviderInfo {
            uuid: p.uuid,
            name: p.name,
            icon: p.icon,
            rdns: p.rdns,
        })
        .collect()
}

/// Fallback: check if window.ethereum exists (legacy EIP-1193)
#[cfg(target_arch = "wasm32")]
pub fn has_legacy_provider() -> bool {
    use wasm_bindgen::prelude::*;

    js_sys::eval("typeof window.ethereum !== 'undefined'")
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Send a JSON-RPC request via legacy window.ethereum
#[cfg(target_arch = "wasm32")]
pub async fn legacy_provider_request(
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let js_code = format!(
        r#"
        (async function() {{
            if (typeof window.ethereum === 'undefined') throw new Error('No provider');
            const result = await window.ethereum.request({{
                method: '{method}',
                params: {params}
            }});
            return JSON.stringify(result);
        }})()
        "#,
        method = method,
        params = params.to_string(),
    );

    let promise = js_sys::eval(&js_code).map_err(|e| format!("{:?}", e))?;
    let promise = js_sys::Promise::from(promise);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("{:?}", e))?;
    let text = result.as_string().ok_or("Expected string result")?;
    serde_json::from_str(&text).map_err(|e| format!("{e}"))
}
