use dioxus::prelude::*;

#[component]
pub fn ConfirmModal(
    message: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_cancel.call(()),
            div {
                class: "modal-dialog",
                onclick: move |e| e.stop_propagation(),
                p { class: "modal-message", "{message}" }
                div { class: "modal-actions",
                    button {
                        class: "modal-btn cancel",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "modal-btn confirm",
                        onclick: move |_| on_confirm.call(()),
                        "Delete"
                    }
                }
            }
        }
    }
}
