use serde::{Deserialize, Serialize};

use super::sheet::{Sheet, SheetId};
use super::table::{TableId, TableModel};

/// Given a base name and a list of existing names, return a unique name.
/// If `base` is already unique, returns it as-is. Otherwise appends " (2)", " (3)", etc.
pub fn unique_name(base: &str, existing: &[&str]) -> String {
    if !existing.iter().any(|n| n.eq_ignore_ascii_case(base)) {
        return base.to_string();
    }
    let mut i = 2u32;
    loop {
        let candidate = format!("{} ({})", base, i);
        if !existing.iter().any(|n| n.eq_ignore_ascii_case(&candidate)) {
            return candidate;
        }
        i += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkbookState {
    pub version: u32,
    pub sheets: Vec<Sheet>,
    pub active_sheet_id: SheetId,
    pub next_sheet_id: SheetId,

    // Legacy field — ignored on new workbooks, kept for migration
    #[serde(default, skip_serializing)]
    pub tables: Vec<TableModel>,
    #[serde(default, skip_serializing)]
    pub active_table_id: TableId,
    #[serde(default, skip_serializing)]
    pub next_table_id: TableId,
}

impl Default for WorkbookState {
    fn default() -> Self {
        let sheet = Sheet::new(1, "Sheet 1".to_string());
        Self {
            version: 2,
            sheets: vec![sheet],
            active_sheet_id: 1,
            next_sheet_id: 2,
            tables: Vec::new(),
            active_table_id: 0,
            next_table_id: 0,
        }
    }
}

impl WorkbookState {
    /// Migrate v1 (flat tables) to v2 (sheets containing tables)
    pub fn migrate_if_needed(&mut self) {
        if self.version < 2 && !self.tables.is_empty() {
            let mut sheet = Sheet {
                id: 1,
                name: "Sheet 1".to_string(),
                tables: std::mem::take(&mut self.tables),
                next_table_id: self.next_table_id,
                active_table_id: self.active_table_id,
            };
            // Ensure tables have header_row set
            for t in &mut sheet.tables {
                if !t.header_row {
                    t.header_row = true;
                }
            }
            self.sheets = vec![sheet];
            self.active_sheet_id = 1;
            self.next_sheet_id = 2;
            self.version = 2;
        }
    }

    pub fn active_sheet(&self) -> Option<&Sheet> {
        self.sheets.iter().find(|s| s.id == self.active_sheet_id)
    }

    pub fn active_sheet_mut(&mut self) -> Option<&mut Sheet> {
        self.sheets
            .iter_mut()
            .find(|s| s.id == self.active_sheet_id)
    }

    pub fn sheet_by_id(&self, id: SheetId) -> Option<&Sheet> {
        self.sheets.iter().find(|s| s.id == id)
    }

    pub fn sheet_by_id_mut(&mut self, id: SheetId) -> Option<&mut Sheet> {
        self.sheets.iter_mut().find(|s| s.id == id)
    }

    fn unique_sheet_name(&self, base: &str) -> String {
        let existing: Vec<&str> = self.sheets.iter().map(|s| s.name.as_str()).collect();
        unique_name(base, &existing)
    }

    pub fn add_sheet(&mut self, name: String) -> SheetId {
        let id = self.next_sheet_id;
        self.next_sheet_id += 1;
        let name = self.unique_sheet_name(&name);
        self.sheets.push(Sheet::new(id, name));
        self.active_sheet_id = id;
        id
    }

    pub fn delete_sheet(&mut self, id: SheetId) {
        if self.sheets.len() <= 1 {
            return;
        }
        self.sheets.retain(|s| s.id != id);
        if self.active_sheet_id == id {
            self.active_sheet_id = self.sheets[0].id;
        }
    }

    pub fn rename_sheet(&mut self, id: SheetId, name: String) {
        let existing: Vec<&str> = self
            .sheets
            .iter()
            .filter(|s| s.id != id)
            .map(|s| s.name.as_str())
            .collect();
        let name = unique_name(&name, &existing);
        if let Some(s) = self.sheets.iter_mut().find(|s| s.id == id) {
            s.name = name;
        }
    }

    /// Find a table by name across all sheets in the active sheet first, then others
    pub fn find_table_by_name(&self, name: &str) -> Option<&TableModel> {
        // Search active sheet first
        if let Some(sheet) = self.active_sheet() {
            if let Some(t) = sheet.table_by_name(name) {
                return Some(t);
            }
        }
        // Then all sheets
        for sheet in &self.sheets {
            if let Some(t) = sheet.table_by_name(name) {
                return Some(t);
            }
        }
        None
    }
}
