use dioxus::prelude::*;

use crate::model::cell::col_index_to_label;
use crate::model::table::TableModel;
use crate::ui::cell_view::CellView;

#[component]
pub fn SheetView(
    table: TableModel,
    selected: Option<(u32, u32)>,
    editing: Option<(u32, u32)>,
    edit_buffer: String,
    clipboard: Option<(u32, u32)>,
    dragging: Option<(u32, u32)>,
    selection_range: Option<(u32, u32, u32, u32)>,
    on_select_cell: EventHandler<(u32, u32)>,
    on_shift_select_cell: EventHandler<(u32, u32)>,
    on_select_row: EventHandler<u32>,
    on_select_col: EventHandler<u32>,
    on_start_edit: EventHandler<(u32, u32)>,
    on_edit_change: EventHandler<String>,
    on_commit_edit: EventHandler<()>,
    on_cancel_edit: EventHandler<()>,
    on_resize_col: EventHandler<(u32, f32)>,
    on_resize_row: EventHandler<(u32, f32)>,
    on_drag_start: EventHandler<(u32, u32)>,
    on_drag_drop: EventHandler<(u32, u32)>,
    on_drag_end: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "sheet-view",
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    on_cancel_edit.call(());
                }
            },
            table { class: "grid-table",
                thead {
                    tr {
                        th { class: "corner-cell" }
                        for c in 0..table.cols {
                            {
                                let header_name = table.col_display_name(c);
                                let fallback = col_index_to_label(c);
                                let is_custom = header_name != fallback;
                                rsx! {
                                    th {
                                        class: if is_custom { "col-header named clickable" } else { "col-header clickable" },
                                        style: "width: {table.col_width(c)}px; min-width: {table.col_width(c)}px;",
                                        onclick: move |_| on_select_col.call(c),
                                        if is_custom {
                                            div { class: "header-custom-name", "{header_name}" }
                                            div { class: "header-letter", "{fallback}" }
                                        } else {
                                            "{fallback}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                tbody {
                    for r in 0..table.rows {
                        tr {
                            key: "row-{r}",
                            {
                                let row_name = table.row_display_name(r);
                                let fallback = (r + 1).to_string();
                                let is_custom = row_name != fallback;
                                rsx! {
                                    td {
                                        class: if is_custom { "row-header named clickable" } else { "row-header clickable" },
                                        style: "height: {table.row_height(r)}px;",
                                        onclick: move |_| on_select_row.call(r),
                                        if is_custom {
                                            div { class: "header-custom-name", "{row_name}" }
                                            div { class: "header-number", "{fallback}" }
                                        } else {
                                            "{fallback}"
                                        }
                                    }
                                }
                            }
                            for c in 0..table.cols {
                                {
                                    let is_selected = selected == Some((c, r))
                                        || selection_range.map(|(mc, mr, xc, xr)| c >= mc && c <= xc && r >= mr && r <= xr).unwrap_or(false);
                                    let is_editing = editing == Some((c, r));
                                    let is_clipboard = clipboard == Some((c, r));
                                    let is_drag_source = dragging == Some((c, r));
                                    let is_header = table.is_header_cell(c, r);
                                    let cell = table.get_cell(c, r);
                                    let display = cell
                                        .map(|c| c.format.format_value(&c.computed))
                                        .unwrap_or_default();
                                    let has_content = cell
                                        .map(|c| !c.source.is_empty())
                                        .unwrap_or(false);
                                    let cell_style = cell
                                        .map(|c| c.format.css_style())
                                        .unwrap_or_default();
                                    let width = table.col_width(c);
                                    let height = table.row_height(r);

                                    rsx! {
                                        CellView {
                                            key: "cell-{c}-{r}",
                                            col: c,
                                            row: r,
                                            display_value: display,
                                            is_selected,
                                            is_editing,
                                            is_clipboard,
                                            is_drag_source,
                                            is_header,
                                            has_content,
                                            edit_buffer: if is_editing { edit_buffer.clone() } else { String::new() },
                                            width,
                                            height,
                                            cell_style,
                                            on_select: move |_| on_select_cell.call((c, r)),
                                            on_shift_select: move |_| on_shift_select_cell.call((c, r)),
                                            on_start_edit: move |_| on_start_edit.call((c, r)),
                                            on_edit_change: move |v: String| on_edit_change.call(v),
                                            on_commit: move |_| on_commit_edit.call(()),
                                            on_cancel: move |_| on_cancel_edit.call(()),
                                            on_drag_start: move |_| on_drag_start.call((c, r)),
                                            on_drag_drop: move |_| on_drag_drop.call((c, r)),
                                            on_drag_end: move |_| on_drag_end.call(()),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
