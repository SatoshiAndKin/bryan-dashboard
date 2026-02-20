use dioxus::prelude::*;

use crate::formula::graph::recalculate_table;
use crate::model::cell::col_index_to_label;
use crate::model::sheet::SheetId;
use crate::model::table::TableId;
use crate::persistence::{load_workbook, save_workbook};
use crate::ui::confirm_modal::ConfirmModal;
use crate::ui::func_sidebar::FuncSidebar;
use crate::ui::grid::SheetView;
use crate::ui::tabs::SheetTabsPanel;

#[derive(Debug, Clone, PartialEq)]
pub enum PendingDelete {
    Sheet(SheetId, String),
    Table(TableId, String),
    Row(TableId, u32),
    Col(TableId, u32),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiState {
    /// (table_id, col, row)
    pub selected: Option<(TableId, u32, u32)>,
    pub editing: Option<(TableId, u32, u32)>,
    pub edit_buffer: String,
    pub clipboard: Option<(TableId, u32, u32)>,
    pub dragging: Option<(TableId, u32, u32)>,
}

#[component]
pub fn WorkbookShell() -> Element {
    let mut workbook = use_signal(|| {
        let mut wb = load_workbook();
        for sheet in &mut wb.sheets {
            sheet.recalculate_all();
        }
        wb
    });
    let mut ui = use_signal(UiState::default);
    let mut pending_delete: Signal<Option<PendingDelete>> = use_signal(|| None);

    let wb = workbook.read();
    let ui_state = ui.read();

    let sheets = wb.sheets.clone();
    let active_sheet_id = wb.active_sheet_id;
    let active_sheet = wb.active_sheet().cloned();
    let editing = ui_state.editing;
    let edit_buffer = ui_state.edit_buffer.clone();
    let selected = ui_state.selected;
    let clipboard = ui_state.clipboard;
    let dragging = ui_state.dragging;
    let show_func_sidebar = editing.is_some() && edit_buffer.starts_with('=');

    let sel_info = selected.map(|(_tid, col, row)| (col, row));
    let sel_table_id = selected.map(|(tid, _, _)| tid);
    let sel_source = selected
        .and_then(|(tid, col, row)| {
            active_sheet
                .as_ref()
                .and_then(|s| s.table_by_id(tid))
                .and_then(|t| t.get_cell(col, row))
                .map(|c| c.source.clone())
        })
        .unwrap_or_default();

    rsx! {
        div {
            class: "workbook-shell",
            tabindex: "0",
            onkeydown: move |e| {
                let ctrl = e.modifiers().meta() || e.modifiers().ctrl();
                if ctrl && e.key() == Key::Character("c".to_string()) {
                    let u = ui.read();
                    if u.editing.is_none() {
                        if let Some(sel) = u.selected {
                            drop(u);
                            ui.write().clipboard = Some(sel);
                        }
                    }
                } else if ctrl && e.key() == Key::Character("v".to_string()) {
                    let u = ui.read();
                    if u.editing.is_none() {
                        if let (Some(from), Some(to)) = (u.clipboard, u.selected) {
                            if from.0 == to.0 {
                                drop(u);
                                let mut wb = workbook.write();
                                if let Some(sheet) = wb.active_sheet_mut() {
                                    if let Some(table) = sheet.table_by_id_mut(from.0) {
                                        table.copy_cell((from.1, from.2), (to.1, to.2));
                                        recalculate_table(table);
                                    }
                                }
                                save_workbook(&wb);
                            }
                        }
                    }
                } else if ctrl && e.key() == Key::Character("x".to_string()) {
                    let u = ui.read();
                    if u.editing.is_none() {
                        if let Some(sel) = u.selected {
                            drop(u);
                            ui.write().clipboard = Some(sel);
                            let mut wb = workbook.write();
                            if let Some(sheet) = wb.active_sheet_mut() {
                                if let Some(table) = sheet.table_by_id_mut(sel.0) {
                                    table.set_cell_source(sel.1, sel.2, String::new());
                                    recalculate_table(table);
                                }
                            }
                            save_workbook(&wb);
                        }
                    }
                } else if !ctrl {
                    // Type-to-edit: start editing when a printable key is pressed on a selected cell
                    let u = ui.read();
                    if u.editing.is_none() {
                        if let Some((tid, col, row)) = u.selected {
                            let ch = match e.key() {
                                Key::Character(ref s) if !s.is_empty() => Some(s.clone()),
                                _ => None,
                            };
                            if let Some(ch) = ch {
                                // Ignore modifier-only or navigation keys
                                if e.key() != Key::Tab
                                    && e.key() != Key::Escape
                                    && e.key() != Key::Enter
                                    && e.key() != Key::Backspace
                                    && e.key() != Key::Delete
                                {
                                    drop(u);
                                    let mut u = ui.write();
                                    u.editing = Some((tid, col, row));
                                    u.edit_buffer = ch;
                                }
                            }
                        }
                    }
                }
            },

            // Sheet tabs (top bar)
            SheetTabsPanel {
                sheets: sheets.clone(),
                active_id: active_sheet_id,
                on_select: move |id: SheetId| {
                    workbook.write().active_sheet_id = id;
                    let mut u = ui.write();
                    u.selected = None;
                    u.editing = None;
                },
                on_add: move |_: ()| {
                    let mut wb = workbook.write();
                    let n = wb.sheets.len() + 1;
                    wb.add_sheet(format!("Sheet {}", n));
                    save_workbook(&wb);
                },
                on_delete: move |id: SheetId| {
                    let wb = workbook.read();
                    let name = wb.sheet_by_id(id)
                        .map(|s| s.name.clone())
                        .unwrap_or_default();
                    drop(wb);
                    pending_delete.set(Some(PendingDelete::Sheet(id, name)));
                },
                on_rename: move |(id, name): (SheetId, String)| {
                    let mut wb = workbook.write();
                    wb.rename_sheet(id, name);
                    save_workbook(&wb);
                },
            }

            // Toolbar
            div { class: "sheet-toolbar",
                // Table management (within active sheet)
                button {
                    class: "toolbar-btn",
                    onclick: move |_| {
                        let mut wb = workbook.write();
                        if let Some(sheet) = wb.active_sheet_mut() {
                            let n = sheet.tables.len() + 1;
                            sheet.add_table(format!("Table {}", n), 6, 5);
                        }
                        save_workbook(&wb);
                    },
                    "+ Table"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| {
                        let mut wb = workbook.write();
                        if let Some(sheet) = wb.active_sheet_mut() {
                            if let Some(t) = sheet.active_table_mut() {
                                t.add_row();
                            }
                        }
                        save_workbook(&wb);
                    },
                    "+ Row"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| {
                        let mut wb = workbook.write();
                        if let Some(sheet) = wb.active_sheet_mut() {
                            if let Some(t) = sheet.active_table_mut() {
                                t.add_col();
                            }
                        }
                        save_workbook(&wb);
                    },
                    "+ Col"
                }
                if let Some((_col, row)) = sel_info {
                    button {
                        class: "toolbar-btn danger",
                        onclick: move |_| {
                            if let Some(tid) = sel_table_id {
                                pending_delete.set(Some(PendingDelete::Row(tid, row)));
                            }
                        },
                        "- Row"
                    }
                }
                if let Some((col, _row)) = sel_info {
                    button {
                        class: "toolbar-btn danger",
                        onclick: move |_| {
                            if let Some(tid) = sel_table_id {
                                pending_delete.set(Some(PendingDelete::Col(tid, col)));
                            }
                        },
                        "- Col"
                    }
                }
                if let Some((col, row)) = sel_info {
                    span { class: "cell-indicator",
                        "{col_index_to_label(col)}{row + 1}"
                    }
                }
                if editing.is_some() {
                    span { class: "formula-bar",
                        "{edit_buffer}"
                    }
                } else if !sel_source.is_empty() {
                    span { class: "formula-bar readonly",
                        "{sel_source}"
                    }
                }
            }

            // Canvas with tables from active sheet
            div { class: "sheet-content",
                div { class: "canvas-area",
                    if let Some(sheet) = &active_sheet {
                        for table in &sheet.tables {
                            {
                                let tid = table.id;
                                let is_active = tid == sheet.active_table_id;
                                let table_name = table.name.clone();
                                let t_selected = selected.and_then(|(t, c, r)| if t == tid { Some((c, r)) } else { None });
                                let t_editing = editing.and_then(|(t, c, r)| if t == tid { Some((c, r)) } else { None });
                                let t_clipboard = clipboard.and_then(|(t, c, r)| if t == tid { Some((c, r)) } else { None });
                                let t_dragging = dragging.and_then(|(t, c, r)| if t == tid { Some((c, r)) } else { None });
                                let t_edit_buffer = if t_editing.is_some() { edit_buffer.clone() } else { String::new() };
                                let can_delete_table = sheet.tables.len() > 1;
                                rsx! {
                                    div {
                                        class: if is_active { "canvas-table active" } else { "canvas-table" },
                                        div { class: "canvas-table-header",
                                            span { class: "canvas-table-name", "{table_name}" }
                                            if can_delete_table {
                                                button {
                                                    class: "canvas-table-delete",
                                                    onclick: move |_| {
                                                        pending_delete.set(Some(PendingDelete::Table(tid, table_name.clone())));
                                                    },
                                                    "x"
                                                }
                                            }
                                        }
                                        SheetView {
                                            table: table.clone(),
                                            selected: t_selected,
                                            editing: t_editing,
                                            edit_buffer: t_edit_buffer,
                                            clipboard: t_clipboard,
                                            dragging: t_dragging,
                                            on_select_cell: move |(col, row): (u32, u32)| {
                                                let mut u = ui.write();
                                                if u.editing.is_some() && u.edit_buffer.starts_with('=') {
                                                    // If clicking a different table, insert TABLE_NAME::REF
                                                    let editing_tid = u.editing.map(|(t, _, _)| t);
                                                    if editing_tid != Some(tid) {
                                                        let wb = workbook.read();
                                                        let tname = wb.active_sheet()
                                                            .and_then(|s| s.table_by_id(tid))
                                                            .map(|t| t.name.clone())
                                                            .unwrap_or_default();
                                                        drop(wb);
                                                        let label = format!(
                                                            "{}::{}{}",
                                                            tname,
                                                            crate::model::cell::col_index_to_label(col),
                                                            row + 1
                                                        );
                                                        u.edit_buffer.push_str(&label);
                                                    } else {
                                                        let label = format!(
                                                            "{}{}",
                                                            crate::model::cell::col_index_to_label(col),
                                                            row + 1
                                                        );
                                                        u.edit_buffer.push_str(&label);
                                                    }
                                                } else {
                                                    {
                                                        let mut wb = workbook.write();
                                                        if let Some(sheet) = wb.active_sheet_mut() {
                                                            sheet.active_table_id = tid;
                                                        }
                                                    }
                                                    u.selected = Some((tid, col, row));
                                                    u.editing = None;
                                                }
                                            },
                                            on_start_edit: move |(col, row): (u32, u32)| {
                                                let source = {
                                                    let wb = workbook.read();
                                                    wb.active_sheet()
                                                        .and_then(|s| s.table_by_id(tid))
                                                        .and_then(|t| t.get_cell(col, row))
                                                        .map(|c| c.source.clone())
                                                        .unwrap_or_default()
                                                };
                                                {
                                                    let mut wb = workbook.write();
                                                    if let Some(sheet) = wb.active_sheet_mut() {
                                                        sheet.active_table_id = tid;
                                                    }
                                                }
                                                let mut u = ui.write();
                                                u.selected = Some((tid, col, row));
                                                u.editing = Some((tid, col, row));
                                                u.edit_buffer = source;
                                            },
                                            on_edit_change: move |val: String| {
                                                ui.write().edit_buffer = val;
                                            },
                                            on_commit_edit: move |_: ()| {
                                                let u = ui.read();
                                                if let Some((etid, col, row)) = u.editing {
                                                    let source = u.edit_buffer.clone();
                                                    drop(u);
                                                    {
                                                        let mut wb = workbook.write();
                                                        if let Some(sheet) = wb.active_sheet_mut() {
                                                            if let Some(table) = sheet.table_by_id_mut(etid) {
                                                                table.set_cell_source(col, row, source);
                                                                recalculate_table(table);
                                                            }
                                                        }
                                                        save_workbook(&wb);
                                                    }
                                                    ui.write().editing = None;
                                                }
                                            },
                                            on_cancel_edit: move |_: ()| {
                                                let mut u = ui.write();
                                                u.editing = None;
                                                u.edit_buffer.clear();
                                            },
                                            on_resize_col: move |(col, width): (u32, f32)| {
                                                let mut wb = workbook.write();
                                                if let Some(sheet) = wb.active_sheet_mut() {
                                                    if let Some(table) = sheet.table_by_id_mut(tid) {
                                                        table.col_widths.insert(col, width);
                                                    }
                                                }
                                                save_workbook(&wb);
                                            },
                                            on_resize_row: move |(row, height): (u32, f32)| {
                                                let mut wb = workbook.write();
                                                if let Some(sheet) = wb.active_sheet_mut() {
                                                    if let Some(table) = sheet.table_by_id_mut(tid) {
                                                        table.row_heights.insert(row, height);
                                                    }
                                                }
                                                save_workbook(&wb);
                                            },
                                            on_drag_start: move |(col, row): (u32, u32)| {
                                                ui.write().dragging = Some((tid, col, row));
                                            },
                                            on_drag_drop: move |(col, row): (u32, u32)| {
                                                let drag_from = ui.read().dragging;
                                                if let Some((ftid, fc, fr)) = drag_from {
                                                    if ftid == tid && (fc, fr) != (col, row) {
                                                        let mut wb = workbook.write();
                                                        if let Some(sheet) = wb.active_sheet_mut() {
                                                            if let Some(table) = sheet.table_by_id_mut(tid) {
                                                                table.move_cell((fc, fr), (col, row));
                                                                recalculate_table(table);
                                                            }
                                                        }
                                                        save_workbook(&wb);
                                                    }
                                                }
                                                let mut u = ui.write();
                                                u.dragging = None;
                                                u.selected = Some((tid, col, row));
                                            },
                                            on_drag_end: move |_: ()| {
                                                ui.write().dragging = None;
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if show_func_sidebar {
                    FuncSidebar {
                        on_insert: move |text: String| {
                            ui.write().edit_buffer.push_str(&text);
                        },
                    }
                }
            }

            // Confirmation modal
            if let Some(pd) = pending_delete() {
                {
                    let msg = match &pd {
                        PendingDelete::Sheet(_, name) => format!("Delete sheet \"{}\"? All tables and data in it will be lost.", name),
                        PendingDelete::Table(_, name) => format!("Delete table \"{}\"? All data in it will be lost.", name),
                        PendingDelete::Row(_, row) => format!("Delete row {}?", row + 1),
                        PendingDelete::Col(_, col) => format!("Delete column {}?", col_index_to_label(*col)),
                    };
                    rsx! {
                        ConfirmModal {
                            message: msg,
                            on_confirm: move |_: ()| {
                                match &pd {
                                    PendingDelete::Sheet(id, _) => {
                                        let id = *id;
                                        let mut wb = workbook.write();
                                        wb.delete_sheet(id);
                                        save_workbook(&wb);
                                        let mut u = ui.write();
                                        u.selected = None;
                                        u.editing = None;
                                    }
                                    PendingDelete::Table(tid, _) => {
                                        let tid = *tid;
                                        let mut wb = workbook.write();
                                        if let Some(sheet) = wb.active_sheet_mut() {
                                            sheet.delete_table(tid);
                                        }
                                        save_workbook(&wb);
                                        let mut u = ui.write();
                                        if u.selected.map(|(t, _, _)| t) == Some(tid) {
                                            u.selected = None;
                                            u.editing = None;
                                        }
                                    }
                                    PendingDelete::Row(tid, row) => {
                                        let tid = *tid;
                                        let row = *row;
                                        let mut wb = workbook.write();
                                        if let Some(sheet) = wb.active_sheet_mut() {
                                            if let Some(table) = sheet.table_by_id_mut(tid) {
                                                table.delete_row(row);
                                                recalculate_table(table);
                                            }
                                        }
                                        save_workbook(&wb);
                                        let mut u = ui.write();
                                        u.selected = None;
                                        u.editing = None;
                                    }
                                    PendingDelete::Col(tid, col) => {
                                        let tid = *tid;
                                        let col = *col;
                                        let mut wb = workbook.write();
                                        if let Some(sheet) = wb.active_sheet_mut() {
                                            if let Some(table) = sheet.table_by_id_mut(tid) {
                                                table.delete_col(col);
                                                recalculate_table(table);
                                            }
                                        }
                                        save_workbook(&wb);
                                        let mut u = ui.write();
                                        u.selected = None;
                                        u.editing = None;
                                    }
                                }
                                pending_delete.set(None);
                            },
                            on_cancel: move |_: ()| {
                                pending_delete.set(None);
                            },
                        }
                    }
                }
            }
        }
    }
}
