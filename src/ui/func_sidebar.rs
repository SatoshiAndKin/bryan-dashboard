use dioxus::prelude::*;

use crate::formula::registry::{BUILTIN_FUNCTIONS, OPERATORS, REFERENCES};

#[component]
pub fn FuncSidebar(on_insert: EventHandler<String>) -> Element {
    rsx! {
        aside { class: "func-sidebar",
            h3 { class: "func-sidebar-title", "Functions" }

            div { class: "func-category",
                h4 { class: "func-category-title", "Functions" }
                for func in BUILTIN_FUNCTIONS {
                    div {
                        class: "func-item",
                        onclick: move |_| on_insert.call(format!("{}(", func.name)),
                        div { class: "func-item-name", "{func.name}" }
                        div { class: "func-item-syntax", "{func.syntax}" }
                        div { class: "func-item-desc", "{func.description}" }
                    }
                }
            }

            div { class: "func-category",
                h4 { class: "func-category-title", "Operators" }
                for op in OPERATORS {
                    div { class: "func-item",
                        div { class: "func-item-name", "{op.name}" }
                        div { class: "func-item-syntax", "{op.syntax}" }
                        div { class: "func-item-desc", "{op.description}" }
                    }
                }
            }

            div { class: "func-category",
                h4 { class: "func-category-title", "References" }
                for r in REFERENCES {
                    div { class: "func-item",
                        div { class: "func-item-name", "{r.name}" }
                        div { class: "func-item-syntax", "{r.syntax}" }
                        div { class: "func-item-desc", "{r.description}" }
                    }
                }
            }
        }
    }
}
