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
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn migrate_if_needed(&mut self) {
        if self.version < 2 && !self.tables.is_empty() {
            let mut sheet = Sheet {
                id: 1,
                name: "Sheet 1".to_string(),
                tables: std::mem::take(&mut self.tables),
                next_table_id: self.next_table_id,
                active_table_id: self.active_table_id,
            };
            // Ensure tables have header rows/cols set for migrated data
            for t in &mut sheet.tables {
                if t.header_rows == 0 {
                    t.header_rows = 1;
                }
                if t.header_cols == 0 {
                    t.header_cols = 1;
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
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_name_no_conflict() {
        assert_eq!(unique_name("Sheet 1", &[]), "Sheet 1");
        assert_eq!(unique_name("Sheet 1", &["Sheet 2"]), "Sheet 1");
    }

    #[test]
    fn test_unique_name_with_conflict() {
        assert_eq!(unique_name("Sheet 1", &["Sheet 1"]), "Sheet 1 (2)");
        assert_eq!(
            unique_name("Sheet 1", &["Sheet 1", "Sheet 1 (2)"]),
            "Sheet 1 (3)"
        );
    }

    #[test]
    fn test_unique_name_case_insensitive() {
        assert_eq!(unique_name("sheet 1", &["Sheet 1"]), "sheet 1 (2)");
    }

    #[test]
    fn test_add_sheet_unique_names() {
        let mut wb = WorkbookState::default();
        wb.add_sheet("Sheet 1".to_string()); // conflict with existing
        assert_eq!(wb.sheets.len(), 2);
        assert_eq!(wb.sheets[1].name, "Sheet 1 (2)");
    }

    #[test]
    fn test_delete_sheet_cannot_delete_last() {
        let mut wb = WorkbookState::default();
        let id = wb.sheets[0].id;
        wb.delete_sheet(id);
        assert_eq!(wb.sheets.len(), 1); // still 1
    }

    #[test]
    fn test_delete_sheet_switches_active() {
        let mut wb = WorkbookState::default();
        let id1 = wb.sheets[0].id;
        let id2 = wb.add_sheet("Sheet 2".to_string());
        wb.active_sheet_id = id1;
        wb.delete_sheet(id1);
        assert_eq!(wb.active_sheet_id, id2);
    }

    #[test]
    fn test_rename_sheet_unique() {
        let mut wb = WorkbookState::default();
        let id2 = wb.add_sheet("Sheet 2".to_string());
        wb.rename_sheet(id2, "Sheet 1".to_string()); // conflict
        assert_eq!(
            wb.sheets.iter().find(|s| s.id == id2).unwrap().name,
            "Sheet 1 (2)"
        );
    }

    #[test]
    fn test_migrate_v1_to_v2() {
        use crate::model::table::TableModel;
        let mut wb = WorkbookState {
            version: 1,
            sheets: Vec::new(),
            active_sheet_id: 0,
            next_sheet_id: 1,
            tables: vec![
                TableModel::new(1, "T1".to_string(), 3, 3),
                TableModel::new(2, "T2".to_string(), 2, 2),
            ],
            active_table_id: 1,
            next_table_id: 3,
        };
        wb.migrate_if_needed();
        assert_eq!(wb.version, 2);
        assert_eq!(wb.sheets.len(), 1);
        assert_eq!(wb.sheets[0].tables.len(), 2);
        assert!(wb.tables.is_empty());
        // Headers should be set to 1 if they were 0
        for t in &wb.sheets[0].tables {
            assert!(t.header_rows >= 1);
            assert!(t.header_cols >= 1);
        }
    }

    #[test]
    fn test_find_table_by_name() {
        let mut wb = WorkbookState::default();
        if let Some(sheet) = wb.active_sheet_mut() {
            sheet.tables[0].name = "Prices".to_string();
        }
        assert!(wb.find_table_by_name("Prices").is_some());
        assert!(wb.find_table_by_name("Nonexistent").is_none());
    }

    #[test]
    fn test_find_table_by_name_active_sheet_first() {
        let mut wb = WorkbookState::default();
        let id2 = wb.add_sheet("Sheet 2".to_string());
        // Both sheets have "Table 1"
        assert_eq!(wb.active_sheet_id, id2);
        let result = wb.find_table_by_name("Table 1");
        assert!(result.is_some());
        // Should return the active sheet's table
        assert_eq!(result.unwrap().id, wb.active_sheet().unwrap().tables[0].id);
    }
}
