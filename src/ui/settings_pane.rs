use dioxus::prelude::*;

use crate::eth::BlockHead;
use crate::model::settings::AppSettings;

#[component]
pub fn SettingsPane(
    settings: AppSettings,
    block_head: Option<BlockHead>,
    on_save: EventHandler<AppSettings>,
    on_close: EventHandler<()>,
) -> Element {
    let mut draft_url = use_signal(|| settings.rpc_url.clone());
    let mut draft_interval = use_signal(|| settings.poll_interval_secs.to_string());
    let mut draft_etherscan_key = use_signal(|| settings.etherscan_api_key.clone());

    let is_ws = {
        let url = draft_url.read().trim().to_lowercase();
        url.starts_with("ws://") || url.starts_with("wss://")
    };

    rsx! {
        div { class: "settings-overlay",
            onclick: move |_| on_close.call(()),
            div { class: "settings-dialog",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "settings-title", "Settings" }

                div { class: "settings-field",
                    label { class: "settings-label", "Ethereum RPC URL" }
                    input {
                        class: "settings-input",
                        r#type: "text",
                        placeholder: "wss://... or https://...",
                        value: "{draft_url}",
                        oninput: move |e| *draft_url.write() = e.value(),
                    }
                    span { class: "settings-hint",
                        if is_ws {
                            "WebSocket detected — will subscribe to newHeads"
                        } else {
                            "HTTP detected — will poll at interval"
                        }
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
                            let new_settings = AppSettings {
                                rpc_url: draft_url.read().trim().to_string(),
                                poll_interval_secs: interval,
                                etherscan_api_key: draft_etherscan_key.read().trim().to_string(),
                            };
                            on_save.call(new_settings);
                        },
                        "Save"
                    }
                }
            }
        }
    }
}
