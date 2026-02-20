use dioxus::prelude::*;

#[component]
pub fn CellView(
    col: u32,
    row: u32,
    display_value: String,
    is_selected: bool,
    is_editing: bool,
    is_clipboard: bool,
    is_drag_source: bool,
    is_header: bool,
    has_content: bool,
    edit_buffer: String,
    width: f32,
    height: f32,
    on_select: EventHandler<()>,
    on_start_edit: EventHandler<()>,
    on_edit_change: EventHandler<String>,
    on_commit: EventHandler<()>,
    on_cancel: EventHandler<()>,
    on_drag_start: EventHandler<()>,
    on_drag_drop: EventHandler<()>,
    on_drag_end: EventHandler<()>,
) -> Element {
    let mut class_parts = vec!["cell"];
    if is_editing {
        class_parts.push("editing");
    } else if is_selected {
        class_parts.push("selected");
    }
    if is_clipboard {
        class_parts.push("clipboard");
    }
    if is_drag_source {
        class_parts.push("drag-source");
    }
    if is_header {
        class_parts.push("header-cell");
    }
    let class = class_parts.join(" ");

    let is_error = display_value.starts_with('#');

    rsx! {
        td {
            class,
            style: "width: {width}px; min-width: {width}px; height: {height}px;",
            draggable: if has_content && !is_editing { "true" } else { "false" },
            onclick: move |_| {
                if !is_editing {
                    on_select.call(());
                }
            },
            ondoubleclick: move |_| {
                on_start_edit.call(());
            },
            ondragstart: move |_| {
                on_drag_start.call(());
            },
            ondragover: move |e| {
                e.prevent_default();
            },
            ondrop: move |_| {
                on_drag_drop.call(());
            },
            ondragend: move |_| {
                on_drag_end.call(());
            },
            if is_editing {
                input {
                    class: "cell-input",
                    value: "{edit_buffer}",
                    onmounted: move |evt| async move {
                        let _ = evt.set_focus(true).await;
                    },
                    oninput: move |e| on_edit_change.call(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter || e.key() == Key::Tab {
                            e.prevent_default();
                            on_commit.call(());
                        } else if e.key() == Key::Escape {
                            on_cancel.call(());
                        }
                    },
                }
            } else if is_error {
                span { class: "cell-error", "{display_value}" }
            } else {
                span { class: "cell-value", "{display_value}" }
            }
        }
    }
}
