use dioxus::prelude::*;

use crate::eth::BlockHead;
use crate::model::settings::{AppSettings, RpcEntry};

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
                                    input {
                                        class: "settings-input",
                                        r#type: "text",
                                        placeholder: "https://rpc1.example.com, https://rpc2.example.com",
                                        value: "{urls}",
                                        oninput: move |e| {
                                            draft_entries.write()[i].urls = e.value();
                                        },
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
