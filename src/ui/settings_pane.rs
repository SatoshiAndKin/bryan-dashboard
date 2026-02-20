use dioxus::prelude::*;
use std::collections::HashMap;

use crate::eth::BlockHead;
use crate::model::settings::{AppSettings, RpcEntry};

#[derive(Clone, PartialEq)]
enum RpcTestResult {
    Testing,
    Ok(u64),
    ChainMismatch { expected: u64, got: u64 },
    Error(String),
}

#[component]
pub fn SettingsPane(
    settings: AppSettings,
    block_head: Option<BlockHead>,
    on_save: EventHandler<AppSettings>,
    on_close: EventHandler<()>,
) -> Element {
    let mut draft_interval = use_signal(|| settings.poll_interval_secs.to_string());
    let mut draft_etherscan_key = use_signal(|| settings.etherscan_api_key.clone());
    let mut draft_entries = use_signal(|| settings.rpc_entries.clone());
    let mut draft_max_retries = use_signal(|| settings.max_retries.to_string());
    let mut draft_retry_backoff = use_signal(|| settings.retry_backoff_ms.to_string());
    let mut test_results: Signal<HashMap<usize, RpcTestResult>> = use_signal(HashMap::new);

    let is_ws = draft_entries
        .read()
        .first()
        .map(|e| e.is_websocket())
        .unwrap_or(false);

    rsx! {
        div { class: "settings-overlay",
            onclick: move |_| on_close.call(()),
            div { class: "settings-dialog",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "settings-title", "Settings" }

                // Chain RPC entries
                div { class: "settings-field",
                    label { class: "settings-label", "Chain RPC Providers" }
                    span { class: "settings-hint",
                        if is_ws {
                            "WebSocket detected — will subscribe to newHeads. Use commas for fallback providers."
                        } else {
                            "Add RPC URLs per chain. Use commas for multiple fallback providers."
                        }
                    }
                    for (i, entry) in draft_entries.read().iter().enumerate() {
                        {
                            let chain_id = entry.chain_id.to_string();
                            let chain_name = entry.chain_name.clone();
                            let urls = entry.urls.clone();
                            rsx! {
                                div { class: "rpc-entry",
                                    div { class: "rpc-entry-row",
                                        input {
                                            class: "settings-input short",
                                            r#type: "number",
                                            placeholder: "Chain ID",
                                            value: "{chain_id}",
                                            oninput: move |e| {
                                                if let Ok(id) = e.value().parse::<u64>() {
                                                    draft_entries.write()[i].chain_id = id;
                                                }
                                            },
                                        }
                                        input {
                                            class: "settings-input",
                                            r#type: "text",
                                            placeholder: "Chain name",
                                            value: "{chain_name}",
                                            oninput: move |e| {
                                                draft_entries.write()[i].chain_name = e.value();
                                            },
                                        }
                                        button {
                                            class: "modal-btn cancel",
                                            onclick: move |_| {
                                                draft_entries.write().remove(i);
                                            },
                                            "x"
                                        }
                                    }
                                    div { class: "rpc-entry-row",
                                        input {
                                            class: "settings-input",
                                            r#type: "text",
                                            placeholder: "https://rpc1.example.com, https://rpc2.example.com",
                                            value: "{urls}",
                                            oninput: move |e| {
                                                test_results.write().remove(&i);
                                                draft_entries.write()[i].urls = e.value();
                                            },
                                        }
                                        button {
                                            class: "toolbar-btn",
                                            onclick: move |_| {
                                                let entries = draft_entries.read().clone();
                                                if let Some(entry) = entries.get(i) {
                                                    let entry = entry.clone();
                                                    let mut results = test_results;
                                                    results.write().insert(i, RpcTestResult::Testing);
                                                    spawn(async move {
                                                        let result = test_rpc_chain_id(&entry).await;
                                                        results.write().insert(i, result);
                                                    });
                                                }
                                            },
                                            "Test"
                                        }
                                    }
                                    {
                                        let result = test_results.read().get(&i).cloned();
                                        match result {
                                            Some(RpcTestResult::Testing) => rsx! {
                                                span { class: "settings-hint", "Testing..." }
                                            },
                                            Some(RpcTestResult::Ok(chain_id)) => rsx! {
                                                span { class: "settings-hint rpc-test-ok",
                                                    "eth_chainId returned {chain_id}"
                                                }
                                            },
                                            Some(RpcTestResult::ChainMismatch { expected, got }) => rsx! {
                                                span { class: "settings-hint rpc-test-err",
                                                    "Chain ID mismatch: expected {expected}, got {got}"
                                                }
                                            },
                                            Some(RpcTestResult::Error(msg)) => rsx! {
                                                span { class: "settings-hint rpc-test-err",
                                                    "{msg}"
                                                }
                                            },
                                            None => rsx! {},
                                        }
                                    }
                                }
                            }
                        }
                    }
                    button {
                        class: "toolbar-btn",
                        onclick: move |_| {
                            draft_entries.write().push(RpcEntry {
                                chain_id: 1,
                                chain_name: String::new(),
                                urls: String::new(),
                            });
                        },
                        "+ Add Chain"
                    }
                }

                if !is_ws {
                    div { class: "settings-field",
                        label { class: "settings-label", "Poll interval (seconds)" }
                        input {
                            class: "settings-input short",
                            r#type: "number",
                            min: "1",
                            max: "3600",
                            value: "{draft_interval}",
                            oninput: move |e| *draft_interval.write() = e.value(),
                        }
                    }
                }

                // Rate limiting / retry settings
                div { class: "settings-field",
                    label { class: "settings-label", "Retry Settings" }
                    div { class: "rpc-entry-row",
                        div { class: "settings-field-inline",
                            span { class: "settings-hint", "Max retries" }
                            input {
                                class: "settings-input short",
                                r#type: "number",
                                min: "0",
                                max: "10",
                                value: "{draft_max_retries}",
                                oninput: move |e| *draft_max_retries.write() = e.value(),
                            }
                        }
                        div { class: "settings-field-inline",
                            span { class: "settings-hint", "Backoff (ms)" }
                            input {
                                class: "settings-input short",
                                r#type: "number",
                                min: "100",
                                max: "30000",
                                value: "{draft_retry_backoff}",
                                oninput: move |e| *draft_retry_backoff.write() = e.value(),
                            }
                        }
                    }
                }

                div { class: "settings-field",
                    label { class: "settings-label", "Etherscan v2 API Key" }
                    input {
                        class: "settings-input",
                        r#type: "text",
                        placeholder: "Your etherscan API key...",
                        value: "{draft_etherscan_key}",
                        oninput: move |e| *draft_etherscan_key.write() = e.value(),
                    }
                    span { class: "settings-hint",
                        "Used for ABI fetching and contract verification"
                    }
                }

                if let Some(bh) = &block_head {
                    div { class: "settings-block-info",
                        span { class: "settings-label", "Latest block" }
                        div { class: "block-info-row",
                            span { class: "block-info-label", "Number:" }
                            span { class: "block-info-value", "{bh.number}" }
                        }
                        div { class: "block-info-row",
                            span { class: "block-info-label", "Hash:" }
                            span { class: "block-info-value mono",
                                "{bh.hash.get(..18).unwrap_or(&bh.hash)}..."
                            }
                        }
                        if bh.timestamp > 0 {
                            div { class: "block-info-row",
                                span { class: "block-info-label", "Timestamp:" }
                                span { class: "block-info-value", "{bh.timestamp}" }
                            }
                        }
                        if let Some(fee) = bh.base_fee {
                            div { class: "block-info-row",
                                span { class: "block-info-label", "Base fee:" }
                                span { class: "block-info-value", "{fee} wei" }
                            }
                        }
                    }
                }

                div { class: "settings-actions",
                    button {
                        class: "modal-btn cancel",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "modal-btn confirm save",
                        onclick: move |_| {
                            let interval = draft_interval.read().parse::<u32>().unwrap_or(10).max(1);
                            let max_retries = draft_max_retries.read().parse::<u32>().unwrap_or(3);
                            let retry_backoff = draft_retry_backoff.read().parse::<u64>().unwrap_or(1000);
                            let mut new_settings = AppSettings::default();
                            new_settings.poll_interval_secs = interval;
                            new_settings.etherscan_api_key = draft_etherscan_key.read().trim().to_string();
                            new_settings.rpc_entries = draft_entries.read().clone();
                            new_settings.max_retries = max_retries;
                            new_settings.retry_backoff_ms = retry_backoff;
                            on_save.call(new_settings);
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn test_rpc_chain_id(entry: &RpcEntry) -> RpcTestResult {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let url = match entry.primary_url() {
        Some(u) => u,
        None => return RpcTestResult::Error("No URL configured".into()),
    };

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_chainId",
        "params": []
    })
    .to_string();

    let window = match web_sys::window() {
        Some(w) => w,
        None => return RpcTestResult::Error("No window object".into()),
    };

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

    let request = match web_sys::Request::new_with_str_and_init(&url, &opts) {
        Ok(r) => r,
        Err(e) => return RpcTestResult::Error(format!("{:?}", e)),
    };
    let _ = request.headers().set("Content-Type", "application/json");

    let resp_val = match JsFuture::from(window.fetch_with_request(&request)).await {
        Ok(v) => v,
        Err(e) => return RpcTestResult::Error(format!("Fetch failed: {:?}", e)),
    };

    let resp: web_sys::Response = match resp_val.dyn_into() {
        Ok(r) => r,
        Err(_) => return RpcTestResult::Error("Invalid response".into()),
    };

    let json_promise = match resp.json() {
        Ok(p) => p,
        Err(e) => return RpcTestResult::Error(format!("{:?}", e)),
    };
    let json_val = match JsFuture::from(json_promise).await {
        Ok(v) => v,
        Err(e) => return RpcTestResult::Error(format!("{:?}", e)),
    };

    let text = match js_sys::JSON::stringify(&json_val) {
        Ok(s) => s.as_string().unwrap_or_default(),
        Err(_) => return RpcTestResult::Error("Failed to stringify response".into()),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return RpcTestResult::Error(format!("Parse error: {e}")),
    };

    if let Some(err) = parsed.get("error") {
        let msg = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown RPC error");
        return RpcTestResult::Error(format!("RPC error: {msg}"));
    }

    let hex = match parsed.get("result").and_then(|v| v.as_str()) {
        Some(h) => h,
        None => return RpcTestResult::Error("No result in response".into()),
    };

    let got = match u64::from_str_radix(hex.trim_start_matches("0x"), 16) {
        Ok(id) => id,
        Err(_) => return RpcTestResult::Error(format!("Invalid chain ID hex: {hex}")),
    };

    if got == entry.chain_id {
        RpcTestResult::Ok(got)
    } else {
        RpcTestResult::ChainMismatch {
            expected: entry.chain_id,
            got,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn test_rpc_chain_id(_entry: &RpcEntry) -> RpcTestResult {
    RpcTestResult::Error("RPC testing only available in browser".into())
}
