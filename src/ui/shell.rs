use dioxus::prelude::*;

use crate::eth::BlockHead;
use crate::formula::graph::{recalculate_table, recalculate_table_with_ctx};
use crate::model::cell::col_index_to_label;
use crate::model::settings::AppSettings;
use crate::model::sheet::SheetId;
use crate::model::table::TableId;
#[cfg(target_arch = "wasm32")]
use crate::persistence::{export_workbook, import_workbook};
use crate::persistence::{load_settings, load_workbook, save_settings, save_workbook};
use crate::ui::buddy::BuddyCharacter;
use crate::ui::confirm_modal::ConfirmModal;
use crate::ui::func_sidebar::FuncSidebar;
use crate::ui::grid::SheetView;
use crate::ui::settings_pane::SettingsPane;
use crate::ui::starfield::Starfield;
use crate::ui::tabs::SheetTabsPanel;

#[derive(Debug, Clone, PartialEq)]
pub enum PendingDelete {
    Sheet(SheetId, String),
    Table(TableId, String),
    Row(TableId, u32),
    Col(TableId, u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UndoEntry {
    pub table_id: TableId,
    pub col: u32,
    pub row: u32,
    pub old_source: String,
    pub new_source: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiState {
    /// (table_id, col, row)
    pub selected: Option<(TableId, u32, u32)>,
    pub editing: Option<(TableId, u32, u32)>,
    /// Which sheet the cell being edited belongs to
    pub editing_sheet_id: Option<SheetId>,
    pub edit_buffer: String,
    pub clipboard: Option<(TableId, u32, u32)>,
    pub dragging: Option<(TableId, u32, u32)>,
    pub undo_stack: Vec<UndoEntry>,
    pub redo_stack: Vec<UndoEntry>,
    /// Toast messages: (message, timestamp_ms)
    pub toasts: Vec<(String, f64)>,
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
    let mut settings = use_signal(load_settings);
    let mut show_settings = use_signal(|| {
        // Prompt settings if no RPC is configured
        let s = load_settings();
        !s.has_rpc()
    });
    let block_head: Signal<Option<BlockHead>> = use_signal(|| None);
    let mut last_saved: Signal<Option<String>> = use_signal(|| None);
    let balance_cache: Signal<std::collections::HashMap<String, String>> =
        use_signal(std::collections::HashMap::new);

    // Ethereum connection effect: reacts to settings changes
    #[cfg(target_arch = "wasm32")]
    {
        let settings_clone = settings.read().clone();
        use_effect(move || {
            let s = settings_clone.clone();
            let bh = block_head;
            spawn(async move {
                connect_eth(s, bh).await;
            });
        });
    }

    // Recalculate all formulas when block_head changes (for BLOCK_NUMBER etc)
    {
        let bh = block_head.read().clone();
        let cache = balance_cache.read().clone();
        let rpc_url = settings.read().effective_rpc_url().unwrap_or_default();
        use_effect(move || {
            let bh_ref = bh.as_ref();
            let pending = std::cell::RefCell::new(Vec::<String>::new());
            let mut wb = workbook.write();
            for sheet in &mut wb.sheets {
                for table in &mut sheet.tables {
                    recalculate_table_with_ctx(table, &[], bh_ref, Some(&cache), Some(&pending));
                }
            }
            drop(wb);
            // Fetch balances for any pending addresses
            let addrs = pending.into_inner();
            #[cfg(target_arch = "wasm32")]
            if !addrs.is_empty() && !rpc_url.is_empty() {
                let url = rpc_url.clone();
                let mut bc = balance_cache;
                spawn(async move {
                    fetch_balances(&url, &addrs, &mut bc).await;
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = (&addrs, &rpc_url);
        });
    }

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

    let active_table_info = active_sheet.as_ref().and_then(|s| {
        s.active_table()
            .map(|t| (t.id, t.header_rows, t.header_cols, t.footer_rows))
    });

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

    let sel_pretty_name = selected.and_then(|(tid, col, row)| {
        active_sheet
            .as_ref()
            .and_then(|s| s.table_by_id(tid))
            .and_then(|t| {
                let col_name = t.col_pretty_name(col);
                let row_name = t.row_pretty_name(row);
                match (col_name, row_name) {
                    (Some(cn), Some(rn)) => Some(format!("{}, {}", cn, rn)),
                    (Some(cn), None) => Some(cn),
                    (None, Some(rn)) => Some(rn),
                    (None, None) => None,
                }
            })
    });

    rsx! {
        div {
            class: "workbook-shell",
            tabindex: "0",
            onkeydown: move |e| {
                let ctrl = e.modifiers().meta() || e.modifiers().ctrl();
                let shift = e.modifiers().shift();
                let is_editing = ui.read().editing.is_some();

                // --- Ctrl shortcuts ---
                if ctrl && e.key() == Key::Character("c".to_string()) && !is_editing {
                    let sel = ui.read().selected;
                    if let Some(sel) = sel {
                        ui.write().clipboard = Some(sel);
                    }
                    return;
                }
                if ctrl && e.key() == Key::Character("v".to_string()) && !is_editing {
                    let (clipboard, selected) = {
                        let u = ui.read();
                        (u.clipboard, u.selected)
                    };
                    if let (Some(from), Some(to)) = (clipboard, selected) {
                        if from.0 == to.0 {
                            let mut wb = workbook.write();
                            if let Some(sheet) = wb.active_sheet_mut() {
                                if let Some(table) = sheet.table_by_id_mut(from.0) {
                                    table.copy_cell((from.1, from.2), (to.1, to.2));
                                    recalculate_table(table);
                                }
                            }
                            save_workbook(&wb); last_saved.set(Some(now_string()));
                        }
                    }
                    return;
                }
                if ctrl && e.key() == Key::Character("x".to_string()) && !is_editing {
                    let sel = ui.read().selected;
                    if let Some(sel) = sel {
                        ui.write().clipboard = Some(sel);
                        let old_source = {
                            let wb = workbook.read();
                            wb.active_sheet()
                                .and_then(|s| s.table_by_id(sel.0))
                                .and_then(|t| t.cells.get(&(sel.1, sel.2)))
                                .map(|c| c.source.clone())
                                .unwrap_or_default()
                        };
                        let mut wb = workbook.write();
                        if let Some(sheet) = wb.active_sheet_mut() {
                            if let Some(table) = sheet.table_by_id_mut(sel.0) {
                                table.set_cell_source(sel.1, sel.2, String::new());
                                recalculate_table(table);
                            }
                        }
                        save_workbook(&wb); last_saved.set(Some(now_string()));
                        if !old_source.is_empty() {
                            let mut u = ui.write();
                            u.undo_stack.push(UndoEntry {
                                table_id: sel.0, col: sel.1, row: sel.2,
                                old_source, new_source: String::new(),
                            });
                            u.redo_stack.clear();
                        }
                    }
                    return;
                }
                // Undo: Ctrl+Z
                if ctrl && !shift && e.key() == Key::Character("z".to_string()) {
                    e.prevent_default();
                    let entry = ui.write().undo_stack.pop();
                    if let Some(entry) = entry {
                        let mut wb = workbook.write();
                        if let Some(sheet) = wb.active_sheet_mut() {
                            if let Some(table) = sheet.table_by_id_mut(entry.table_id) {
                                table.set_cell_source(entry.col, entry.row, entry.old_source.clone());
                                recalculate_table(table);
                            }
                        }
                        save_workbook(&wb); last_saved.set(Some(now_string()));
                        ui.write().redo_stack.push(entry);
                    }
                    return;
                }
                // Redo: Ctrl+Shift+Z or Ctrl+Y
                if (ctrl && shift && e.key() == Key::Character("z".to_string()))
                    || (ctrl && e.key() == Key::Character("y".to_string()))
                {
                    e.prevent_default();
                    let entry = ui.write().redo_stack.pop();
                    if let Some(entry) = entry {
                        let mut wb = workbook.write();
                        if let Some(sheet) = wb.active_sheet_mut() {
                            if let Some(table) = sheet.table_by_id_mut(entry.table_id) {
                                table.set_cell_source(entry.col, entry.row, entry.new_source.clone());
                                recalculate_table(table);
                            }
                        }
                        save_workbook(&wb); last_saved.set(Some(now_string()));
                        ui.write().undo_stack.push(entry);
                    }
                    return;
                }

                if ctrl { return; }

                // --- Non-editing navigation ---
                let selected_cell = if !is_editing { ui.read().selected } else { None };
                if let Some((tid, col, row)) = selected_cell {
                        let table_bounds = {
                            let wb = workbook.read();
                            wb.active_sheet()
                                .and_then(|s| s.table_by_id(tid))
                                .map(|t| (t.cols, t.rows))
                        };
                        if let Some((max_c, max_r)) = table_bounds {
                            let (new_col, new_row) = match e.key() {
                                Key::ArrowUp => (col, row.saturating_sub(1)),
                                Key::ArrowDown => (col, (row + 1).min(max_r - 1)),
                                Key::ArrowLeft => (col.saturating_sub(1), row),
                                Key::ArrowRight => ((col + 1).min(max_c - 1), row),
                                Key::Tab if shift => (col.saturating_sub(1), row),
                                Key::Tab => ((col + 1).min(max_c - 1), row),
                                Key::Enter => (col, (row + 1).min(max_r - 1)),
                                Key::Backspace | Key::Delete => {
                                    e.prevent_default();
                                    let old_source = {
                                        let wb = workbook.read();
                                        wb.active_sheet()
                                            .and_then(|s| s.table_by_id(tid))
                                            .and_then(|t| t.cells.get(&(col, row)))
                                            .map(|c| c.source.clone())
                                            .unwrap_or_default()
                                    };
                                    if !old_source.is_empty() {
                                        let mut wb = workbook.write();
                                        if let Some(sheet) = wb.active_sheet_mut() {
                                            if let Some(table) = sheet.table_by_id_mut(tid) {
                                                table.set_cell_source(col, row, String::new());
                                                recalculate_table(table);
                                            }
                                        }
                                        save_workbook(&wb); last_saved.set(Some(now_string()));
                                        let mut u = ui.write();
                                        u.undo_stack.push(UndoEntry {
                                            table_id: tid, col, row,
                                            old_source, new_source: String::new(),
                                        });
                                        u.redo_stack.clear();
                                    }
                                    return;
                                }
                                _ => {
                                    // Type-to-edit
                                    let ch = match e.key() {
                                        Key::Character(ref s) if !s.is_empty() => Some(s.clone()),
                                        _ => None,
                                    };
                                    if let Some(ch) = ch {
                                        if e.key() != Key::Escape {
                                            e.prevent_default();
                                            let sid = workbook.read().active_sheet_id;
                                            let mut u = ui.write();
                                            u.editing = Some((tid, col, row));
                                            u.editing_sheet_id = Some(sid);
                                            u.edit_buffer = ch;
                                        }
                                    }
                                    return;
                                }
                            };
                            if (new_col, new_row) != (col, row) {
                                e.prevent_default();
                                ui.write().selected = Some((tid, new_col, new_row));
                            }
                        }
                    }
            },

            Starfield {}
            BuddyCharacter {}

            // Sheet tabs (top bar) + last saved timestamp
            div { class: "tabs-row",
            SheetTabsPanel {
                sheets: sheets.clone(),
                active_id: active_sheet_id,
                on_select: move |id: SheetId| {
                    workbook.write().active_sheet_id = id;
                    let u = ui.read();
                    if u.editing.is_none() {
                        drop(u);
                        let mut u = ui.write();
                        u.selected = None;
                    }
                },
                on_add: move |_: ()| {
                    let mut wb = workbook.write();
                    let n = wb.sheets.len() + 1;
                    wb.add_sheet(format!("Sheet {}", n));
                    save_workbook(&wb); last_saved.set(Some(now_string()));
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
                    save_workbook(&wb); last_saved.set(Some(now_string()));
                },
            }
            if let Some(ts) = last_saved() {
                span { class: "last-saved", "Saved {ts}" }
            }
            } // end tabs-row

            // Toolbar
            div { class: "sheet-toolbar",
                button {
                    class: "toolbar-btn",
                    onclick: move |_| show_settings.set(true),
                    "Settings"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let wb = workbook.read();
                            if let Some(json) = export_workbook(&wb) {
                                download_file("workbook.json", &json);
                            }
                        }
                    },
                    "Export"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        trigger_file_import(workbook);
                    },
                    "Import"
                }
                span { class: "toolbar-separator" }
                // Table management (within active sheet)
                button {
                    class: "toolbar-btn",
                    onclick: move |_| {
                        let mut wb = workbook.write();
                        if let Some(sheet) = wb.active_sheet_mut() {
                            let n = sheet.tables.len() + 1;
                            sheet.add_table(format!("Table {}", n), 6, 5);
                        }
                        save_workbook(&wb); last_saved.set(Some(now_string()));
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
                        save_workbook(&wb); last_saved.set(Some(now_string()));
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
                        save_workbook(&wb); last_saved.set(Some(now_string()));
                    },
                    "+ Col"
                }
                // Header / footer config for active table
                if let Some((_atid, h_rows, h_cols, f_rows)) = active_table_info {
                    span { class: "toolbar-separator" }
                    div { class: "toolbar-stepper",
                        span { class: "stepper-label", "H-Rows" }
                        button {
                            class: "stepper-btn",
                            disabled: h_rows == 0,
                            onclick: move |_| {
                                let mut wb = workbook.write();
                                if let Some(sheet) = wb.active_sheet_mut() {
                                    if let Some(t) = sheet.active_table_mut() {
                                        t.header_rows = t.header_rows.saturating_sub(1);
                                    }
                                }
                                save_workbook(&wb); last_saved.set(Some(now_string()));
                            },
                            "-"
                        }
                        span { class: "stepper-value", "{h_rows}" }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let mut wb = workbook.write();
                                if let Some(sheet) = wb.active_sheet_mut() {
                                    if let Some(t) = sheet.active_table_mut() {
                                        if t.header_rows < t.rows.saturating_sub(t.footer_rows) {
                                            t.header_rows += 1;
                                        }
                                    }
                                }
                                save_workbook(&wb); last_saved.set(Some(now_string()));
                            },
                            "+"
                        }
                    }
                    div { class: "toolbar-stepper",
                        span { class: "stepper-label", "H-Cols" }
                        button {
                            class: "stepper-btn",
                            disabled: h_cols == 0,
                            onclick: move |_| {
                                let mut wb = workbook.write();
                                if let Some(sheet) = wb.active_sheet_mut() {
                                    if let Some(t) = sheet.active_table_mut() {
                                        t.header_cols = t.header_cols.saturating_sub(1);
                                    }
                                }
                                save_workbook(&wb); last_saved.set(Some(now_string()));
                            },
                            "-"
                        }
                        span { class: "stepper-value", "{h_cols}" }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let mut wb = workbook.write();
                                if let Some(sheet) = wb.active_sheet_mut() {
                                    if let Some(t) = sheet.active_table_mut() {
                                        if t.header_cols < t.cols {
                                            t.header_cols += 1;
                                        }
                                    }
                                }
                                save_workbook(&wb); last_saved.set(Some(now_string()));
                            },
                            "+"
                        }
                    }
                    div { class: "toolbar-stepper",
                        span { class: "stepper-label", "Footers" }
                        button {
                            class: "stepper-btn",
                            disabled: f_rows == 0,
                            onclick: move |_| {
                                let mut wb = workbook.write();
                                if let Some(sheet) = wb.active_sheet_mut() {
                                    if let Some(t) = sheet.active_table_mut() {
                                        t.footer_rows = t.footer_rows.saturating_sub(1);
                                    }
                                }
                                save_workbook(&wb); last_saved.set(Some(now_string()));
                            },
                            "-"
                        }
                        span { class: "stepper-value", "{f_rows}" }
                        button {
                            class: "stepper-btn",
                            onclick: move |_| {
                                let mut wb = workbook.write();
                                if let Some(sheet) = wb.active_sheet_mut() {
                                    if let Some(t) = sheet.active_table_mut() {
                                        if t.footer_rows < t.rows.saturating_sub(t.header_rows) {
                                            t.footer_rows += 1;
                                        }
                                    }
                                }
                                save_workbook(&wb); last_saved.set(Some(now_string()));
                            },
                            "+"
                        }
                    }
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
            }

            // Formula bar — full width row
            div { class: "formula-bar-row",
                if let Some((col, row)) = sel_info {
                    span { class: "cell-indicator",
                        "{col_index_to_label(col)}{row + 1}"
                    }
                    if let Some(ref pretty) = sel_pretty_name {
                        span { class: "cell-indicator-name", "{pretty}" }
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
                                                    let editing_tid = u.editing.map(|(t, _, _)| t);
                                                    let editing_sid = u.editing_sheet_id;
                                                    let wb = workbook.read();
                                                    let current_sid = wb.active_sheet_id;
                                                    let cross_sheet = editing_sid.is_some() && editing_sid != Some(current_sid);
                                                    let cross_table = editing_tid != Some(tid);

                                                    if cross_sheet {
                                                        // Insert SHEET::TABLE::CELL
                                                        let sname = wb.active_sheet()
                                                            .map(|s| s.name.clone())
                                                            .unwrap_or_default();
                                                        let tname = wb.active_sheet()
                                                            .and_then(|s| s.table_by_id(tid))
                                                            .map(|t| t.name.clone())
                                                            .unwrap_or_default();
                                                        drop(wb);
                                                        let label = format!(
                                                            "{}::{}::{}{}",
                                                            sname, tname,
                                                            crate::model::cell::col_index_to_label(col),
                                                            row + 1
                                                        );
                                                        u.edit_buffer.push_str(&label);
                                                    } else if cross_table {
                                                        // Insert TABLE::CELL
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
                                                        drop(wb);
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
                                                    u.editing_sheet_id = None;
                                                }
                                            },
                                            on_start_edit: move |(col, row): (u32, u32)| {
                                                let (source, sid) = {
                                                    let wb = workbook.read();
                                                    let sid = wb.active_sheet_id;
                                                    let src = wb.active_sheet()
                                                        .and_then(|s| s.table_by_id(tid))
                                                        .and_then(|t| t.get_cell(col, row))
                                                        .map(|c| c.source.clone())
                                                        .unwrap_or_default();
                                                    (src, sid)
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
                                                u.editing_sheet_id = Some(sid);
                                                u.edit_buffer = source;
                                            },
                                            on_edit_change: move |val: String| {
                                                ui.write().edit_buffer = val;
                                            },
                                            on_commit_edit: move |_: ()| {
                                                let u = ui.read();
                                                if let Some((etid, col, row)) = u.editing {
                                                    let new_source = u.edit_buffer.clone();
                                                    let esid = u.editing_sheet_id;
                                                    drop(u);
                                                    let old_source = {
                                                        let wb = workbook.read();
                                                        let target_sid = esid.unwrap_or(wb.active_sheet_id);
                                                        wb.sheets.iter().find(|s| s.id == target_sid)
                                                            .and_then(|s| s.table_by_id(etid))
                                                            .and_then(|t| t.cells.get(&(col, row)))
                                                            .map(|c| c.source.clone())
                                                            .unwrap_or_default()
                                                    };
                                                    let max_row = {
                                                        let wb = workbook.read();
                                                        let target_sid = esid.unwrap_or(wb.active_sheet_id);
                                                        wb.sheets.iter().find(|s| s.id == target_sid)
                                                            .and_then(|s| s.table_by_id(etid))
                                                            .map(|t| t.rows)
                                                            .unwrap_or(1)
                                                    };
                                                    {
                                                        let mut wb = workbook.write();
                                                        let target_sid = esid.unwrap_or(wb.active_sheet_id);
                                                        if let Some(sheet) = wb.sheets.iter_mut().find(|s| s.id == target_sid) {
                                                            if let Some(table) = sheet.table_by_id_mut(etid) {
                                                                table.set_cell_source(col, row, new_source.clone());
                                                                recalculate_table(table);
                                                            }
                                                        }
                                                        save_workbook(&wb); last_saved.set(Some(now_string()));
                                                    }
                                                    let mut u = ui.write();
                                                    u.editing = None;
                                                    u.editing_sheet_id = None;
                                                    // Move selection down after commit
                                                    u.selected = Some((etid, col, (row + 1).min(max_row - 1)));
                                                    // Record undo
                                                    if old_source != new_source {
                                                        u.undo_stack.push(UndoEntry {
                                                            table_id: etid, col, row,
                                                            old_source, new_source,
                                                        });
                                                        u.redo_stack.clear();
                                                    }
                                                }
                                            },
                                            on_cancel_edit: move |_: ()| {
                                                let mut u = ui.write();
                                                u.editing = None;
                                                u.editing_sheet_id = None;
                                                u.edit_buffer.clear();
                                            },
                                            on_resize_col: move |(col, width): (u32, f32)| {
                                                let mut wb = workbook.write();
                                                if let Some(sheet) = wb.active_sheet_mut() {
                                                    if let Some(table) = sheet.table_by_id_mut(tid) {
                                                        table.col_widths.insert(col, width);
                                                    }
                                                }
                                                save_workbook(&wb); last_saved.set(Some(now_string()));
                                            },
                                            on_resize_row: move |(row, height): (u32, f32)| {
                                                let mut wb = workbook.write();
                                                if let Some(sheet) = wb.active_sheet_mut() {
                                                    if let Some(table) = sheet.table_by_id_mut(tid) {
                                                        table.row_heights.insert(row, height);
                                                    }
                                                }
                                                save_workbook(&wb); last_saved.set(Some(now_string()));
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
                                                        save_workbook(&wb); last_saved.set(Some(now_string()));
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

            // Settings pane
            if show_settings() {
                SettingsPane {
                    settings: settings.read().clone(),
                    block_head: block_head.read().clone(),
                    on_save: move |new_settings: AppSettings| {
                        save_settings(&new_settings);
                        settings.set(new_settings);
                        show_settings.set(false);
                    },
                    on_close: move |_: ()| {
                        show_settings.set(false);
                    },
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
                                        save_workbook(&wb); last_saved.set(Some(now_string()));
                                        let mut u = ui.write();
                                        u.selected = None;
                                        u.editing = None;
                                        u.editing_sheet_id = None;
                                    }
                                    PendingDelete::Table(tid, _) => {
                                        let tid = *tid;
                                        let mut wb = workbook.write();
                                        if let Some(sheet) = wb.active_sheet_mut() {
                                            sheet.delete_table(tid);
                                        }
                                        save_workbook(&wb); last_saved.set(Some(now_string()));
                                        let mut u = ui.write();
                                        if u.selected.map(|(t, _, _)| t) == Some(tid) {
                                            u.selected = None;
                                            u.editing = None;
                                            u.editing_sheet_id = None;
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
                                        save_workbook(&wb); last_saved.set(Some(now_string()));
                                        let mut u = ui.write();
                                        u.selected = None;
                                        u.editing = None;
                                        u.editing_sheet_id = None;
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
                                        save_workbook(&wb); last_saved.set(Some(now_string()));
                                        let mut u = ui.write();
                                        u.selected = None;
                                        u.editing = None;
                                        u.editing_sheet_id = None;
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

/// Download a text file in the browser
#[cfg(target_arch = "wasm32")]
fn download_file(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(content));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("application/json");
    let blob = match web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts) {
        Ok(b) => b,
        Err(_) => return,
    };
    let url = match web_sys::Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    let a: web_sys::HtmlAnchorElement = match document.create_element("a") {
        Ok(el) => match el.dyn_into() {
            Ok(a) => a,
            Err(_) => return,
        },
        Err(_) => return,
    };
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    let _ = web_sys::Url::revoke_object_url(&url);
}

/// Trigger a file input dialog for import
#[cfg(target_arch = "wasm32")]
fn trigger_file_import(mut workbook: Signal<crate::model::WorkbookState>) {
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    let input: web_sys::HtmlInputElement = match document.create_element("input") {
        Ok(el) => match el.dyn_into() {
            Ok(i) => i,
            Err(_) => return,
        },
        Err(_) => return,
    };
    input.set_type("file");
    input.set_accept(".json");

    let input_clone = input.clone();
    let onchange = Closure::<dyn FnMut()>::new(move || {
        let files = match input_clone.files() {
            Some(f) => f,
            None => return,
        };
        let file = match files.get(0) {
            Some(f) => f,
            None => return,
        };
        let reader = match web_sys::FileReader::new() {
            Ok(r) => r,
            Err(_) => return,
        };
        let _ = reader.read_as_text(&file);
        let reader_clone = reader.clone();
        let onload = Closure::<dyn FnMut()>::new(move || {
            let result = match reader_clone.result() {
                Ok(r) => r,
                Err(_) => return,
            };
            let text = match result.as_string() {
                Some(t) => t,
                None => return,
            };
            match import_workbook(&text) {
                Ok(mut wb) => {
                    for sheet in &mut wb.sheets {
                        sheet.recalculate_all();
                    }
                    save_workbook(&wb);
                    workbook.set(wb);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Import error: {e}").into());
                }
            }
        });
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
    });
    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    onchange.forget();
    input.click();
}

/// Connect to Ethereum RPC and update block_head signal
#[cfg(target_arch = "wasm32")]
async fn connect_eth(settings: AppSettings, mut block_head: Signal<Option<BlockHead>>) {
    if !settings.has_rpc() {
        block_head.set(None);
        return;
    }

    let url = match settings.effective_rpc_url() {
        Some(u) => u,
        None => return,
    };
    if settings.is_websocket() {
        connect_eth_ws(&url, block_head).await;
    } else {
        poll_eth_http(&url, settings.poll_interval_secs, block_head).await;
    }
}

#[cfg(target_arch = "wasm32")]
async fn connect_eth_ws(url: &str, mut block_head: Signal<Option<BlockHead>>) {
    use crate::eth::parse_block_head;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;

    let ws = match web_sys::WebSocket::new(url) {
        Ok(ws) => ws,
        Err(e) => {
            web_sys::console::error_1(&format!("WebSocket connect failed: {:?}", e).into());
            return;
        }
    };
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    // On open: send eth_subscribe for newHeads
    let ws_clone = ws.clone();
    let onopen = Closure::<dyn FnMut()>::new(move || {
        let subscribe_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        });
        let _ = ws_clone.send_with_str(&subscribe_msg.to_string());
    });
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    // On message: parse subscription notification
    let onmessage =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
            if let Some(text) = e.data().as_string() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Subscription notifications come as {"params": {"result": {...}}}
                    if let Some(result) = val.get("params").and_then(|p| p.get("result")) {
                        if let Some(bh) = parse_block_head(result) {
                            block_head.set(Some(bh));
                        }
                    }
                    // Initial subscription response or other messages — try result directly
                    if let Some(result) = val.get("result") {
                        if let Some(bh) = parse_block_head(result) {
                            block_head.set(Some(bh));
                        }
                    }
                }
            }
        });
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onerror = Closure::<dyn FnMut(web_sys::ErrorEvent)>::new(move |e: web_sys::ErrorEvent| {
        web_sys::console::error_1(&format!("WebSocket error: {:?}", e.message()).into());
    });
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    // Keep alive — don't drop. The closures keep a reference to the ws.
    // We use a pending future that never resolves to keep the task alive.
    let (_, rx) = futures_channel::oneshot::channel::<()>();
    let _ = rx.await;
}

#[cfg(target_arch = "wasm32")]
async fn poll_eth_http(url: &str, interval_secs: u32, mut block_head: Signal<Option<BlockHead>>) {
    use crate::eth::parse_block_head;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getBlockByNumber",
        "params": ["latest", false]
    })
    .to_string();

    loop {
        match fetch_json_rpc(url, &body).await {
            Ok(val) => {
                if let Some(result) = val.get("result") {
                    if let Some(bh) = parse_block_head(result) {
                        block_head.set(Some(bh));
                    }
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("HTTP poll error: {e}").into());
            }
        }
        gloo_timers::future::sleep(std::time::Duration::from_secs(interval_secs as u64)).await;
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_json_rpc(url: &str, body: &str) -> Result<serde_json::Value, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("no window")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(body));

    let request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{:?}", e))?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("{:?}", e))?;

    let resp_val = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{:?}", e))?;

    let resp: web_sys::Response = resp_val.dyn_into().map_err(|_| "not a Response")?;
    let json_promise = resp.json().map_err(|e| format!("{:?}", e))?;
    let json_val = JsFuture::from(json_promise)
        .await
        .map_err(|e| format!("{:?}", e))?;

    let text = js_sys::JSON::stringify(&json_val)
        .map_err(|e| format!("{:?}", e))?
        .as_string()
        .ok_or("stringify failed")?;

    serde_json::from_str(&text).map_err(|e| format!("{e}"))
}

/// Fetch JSON-RPC with retry logic (exponential backoff).
#[cfg(target_arch = "wasm32")]
async fn fetch_json_rpc_with_retry(
    url: &str,
    body: &str,
    max_retries: u32,
    backoff_ms: u64,
) -> Result<serde_json::Value, String> {
    let mut last_err = String::new();
    for attempt in 0..=max_retries {
        match fetch_json_rpc(url, body).await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = e;
                if attempt < max_retries {
                    let delay = backoff_ms * (1u64 << attempt.min(5));
                    gloo_timers::future::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }
    Err(format!(
        "all {} retries failed: {}",
        max_retries + 1,
        last_err
    ))
}

/// Fetch JSON-RPC with fallback across multiple URLs. Tries each URL in order,
/// with retry logic per URL.
#[cfg(target_arch = "wasm32")]
async fn fetch_json_rpc_with_fallback(
    urls: &[String],
    body: &str,
    max_retries: u32,
    backoff_ms: u64,
) -> Result<serde_json::Value, String> {
    if urls.is_empty() {
        return Err("no URLs provided".to_string());
    }
    let mut last_err = String::new();
    for url in urls {
        match fetch_json_rpc_with_retry(url, body, max_retries, backoff_ms).await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = format!("{}: {}", url, e);
            }
        }
    }
    Err(format!("all providers failed: {}", last_err))
}

#[cfg(target_arch = "wasm32")]
async fn fetch_balances(
    url: &str,
    addrs: &[String],
    cache: &mut Signal<std::collections::HashMap<String, String>>,
) {
    for addr in addrs {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getBalance",
            "params": [addr, "latest"]
        })
        .to_string();
        match fetch_json_rpc(url, &body).await {
            Ok(val) => {
                if let Some(result) = val.get("result").and_then(|v| v.as_str()) {
                    cache.write().insert(addr.clone(), result.to_string());
                }
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("eth_getBalance failed for {}: {}", addr, e).into(),
                );
            }
        }
    }
}

fn now_string() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let d = js_sys::Date::new_0();
        let h = d.get_hours();
        let m = d.get_minutes();
        let s = d.get_seconds();
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "now".to_string()
    }
}
