use dioxus::prelude::*;

use crate::model::sheet::{Sheet, SheetId};

#[component]
pub fn SheetTabsPanel(
    sheets: Vec<Sheet>,
    active_id: SheetId,
    on_select: EventHandler<SheetId>,
    on_add: EventHandler<()>,
    on_delete: EventHandler<SheetId>,
    on_rename: EventHandler<(SheetId, String)>,
) -> Element {
    let mut renaming: Signal<Option<SheetId>> = use_signal(|| None);
    let mut rename_buf: Signal<String> = use_signal(String::new);

    rsx! {
        div { class: "table-tabs",
            for sheet in &sheets {
                {
                    let sid = sheet.id;
                    let is_active = sid == active_id;
                    let sname = sheet.name.clone();
                    rsx! {
                        div {
                            class: if is_active { "tab active" } else { "tab" },
                            onclick: move |_| on_select.call(sid),
                            ondoubleclick: move |_| {
                                renaming.set(Some(sid));
                                rename_buf.set(sname.clone());
                            },
                            if renaming() == Some(sid) {
                                input {
                                    class: "tab-rename-input",
                                    value: "{rename_buf}",
                                    oninput: move |e| rename_buf.set(e.value()),
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter {
                                            on_rename.call((sid, rename_buf()));
                                            renaming.set(None);
                                        } else if e.key() == Key::Escape {
                                            renaming.set(None);
                                        }
                                    },
                                }
                            } else {
                                span { "{sheet.name}" }
                            }
                            if sheets.len() > 1 {
                                button {
                                    class: "tab-delete",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        on_delete.call(sid);
                                    },
                                    "x"
                                }
                            }
                        }
                    }
                }
            }
            button {
                class: "tab add-tab",
                onclick: move |_| on_add.call(()),
                "+"
            }
        }
    }
}
