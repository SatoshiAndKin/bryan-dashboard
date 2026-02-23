use serde::{Deserialize, Serialize};

use super::table::{TableId, TableModel};
use super::workbook::unique_name;
use crate::formula::graph::recalculate_table_with_siblings;

pub type SheetId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sheet {
    pub id: SheetId,
    pub name: String,
    pub tables: Vec<TableModel>,
    pub next_table_id: TableId,
    /// Which table on this sheet is currently focused
    #[serde(default)]
    pub active_table_id: TableId,
}

impl Sheet {
    pub fn new(id: SheetId, name: String) -> Self {
        let mut table = TableModel::new(1, "Table 1".to_string(), 6, 5);
        table.canvas_x = 16.0;
        table.canvas_y = 16.0;
        Self {
            id,
            name,
            tables: vec![table],
            next_table_id: 2,
            active_table_id: 1,
        }
    }

    pub fn active_table(&self) -> Option<&TableModel> {
        self.tables.iter().find(|t| t.id == self.active_table_id)
    }

    pub fn active_table_mut(&mut self) -> Option<&mut TableModel> {
        self.tables
            .iter_mut()
            .find(|t| t.id == self.active_table_id)
    }

    pub fn table_by_id(&self, id: TableId) -> Option<&TableModel> {
        self.tables.iter().find(|t| t.id == id)
    }

    pub fn table_by_id_mut(&mut self, id: TableId) -> Option<&mut TableModel> {
        self.tables.iter_mut().find(|t| t.id == id)
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn table_by_name(&self, name: &str) -> Option<&TableModel> {
        self.tables.iter().find(|t| t.name == name)
    }

    fn unique_table_name(&self, base: &str) -> String {
        let existing: Vec<&str> = self.tables.iter().map(|t| t.name.as_str()).collect();
        unique_name(base, &existing)
    }

    pub fn add_table(&mut self, name: String, rows: u32, cols: u32) -> TableId {
        let id = self.next_table_id;
        self.next_table_id += 1;
        let name = self.unique_table_name(&name);
        let mut table = TableModel::new(id, name, rows, cols);
        let max_y = self
            .tables
            .iter()
            .map(|t| t.canvas_y + t.pixel_height() + 16.0)
            .fold(16.0f32, f32::max);
        table.canvas_x = 16.0;
        table.canvas_y = max_y;
        table.header_rows = 1;
        table.header_cols = 1;
        self.tables.push(table);
        self.active_table_id = id;
        id
    }

    pub fn delete_table(&mut self, id: TableId) {
        if self.tables.len() <= 1 {
            return;
        }
        self.tables.retain(|t| t.id != id);
        if self.active_table_id == id {
            self.active_table_id = self.tables[0].id;
        }
    }

    pub fn rename_table(&mut self, id: TableId, name: String) {
        let existing: Vec<&str> = self
            .tables
            .iter()
            .filter(|t| t.id != id)
            .map(|t| t.name.as_str())
            .collect();
        let name = unique_name(&name, &existing);
        if let Some(t) = self.tables.iter_mut().find(|t| t.id == id) {
            t.name = name;
        }
    }

    pub fn recalculate_all(&mut self) {
        self.recalculate_all_with_siblings();
    }

    pub fn recalculate_all_with_siblings(&mut self) {
        // Two-pass: first pass calculates each table with snapshot of siblings
        // This handles cross-table references by giving each table a view of others
        for i in 0..self.tables.len() {
            let siblings: Vec<_> = self
                .tables
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, t)| t.clone())
                .collect();
            recalculate_table_with_siblings(&mut self.tables[i], &siblings);
        }
    }

    pub fn recalculate_dependents(&mut self, changed_table_id: TableId) {
        // After a table changes, recalculate all other tables that might reference it
        let snapshot: Vec<_> = self.tables.clone();
        for table in &mut self.tables {
            if table.id == changed_table_id {
                continue;
            }
            // Check if any cell in this table has a cross-table ref
            let has_cross_ref = table.cells.values().any(|c| c.source.contains("::"));
            if has_cross_ref {
                let siblings: Vec<_> = snapshot
                    .iter()
                    .filter(|t| t.id != table.id)
                    .cloned()
                    .collect();
                recalculate_table_with_siblings(table, &siblings);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::graph::recalculate_table;

    #[test]
    fn test_new_sheet_has_one_table() {
        let sheet = Sheet::new(1, "S1".to_string());
        assert_eq!(sheet.tables.len(), 1);
        assert_eq!(sheet.active_table_id, 1);
    }

    #[test]
    fn test_add_table() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        let id = sheet.add_table("Table 2".to_string(), 4, 3);
        assert_eq!(sheet.tables.len(), 2);
        assert_eq!(sheet.active_table_id, id);
    }

    #[test]
    fn test_add_table_unique_name() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        sheet.add_table("Table 1".to_string(), 4, 3);
        assert_eq!(sheet.tables[1].name, "Table 1 (2)");
    }

    #[test]
    fn test_delete_table_cannot_delete_last() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        let id = sheet.tables[0].id;
        sheet.delete_table(id);
        assert_eq!(sheet.tables.len(), 1);
    }

    #[test]
    fn test_delete_table_switches_active() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        let id1 = sheet.tables[0].id;
        let id2 = sheet.add_table("T2".to_string(), 3, 3);
        sheet.active_table_id = id1;
        sheet.delete_table(id1);
        assert_eq!(sheet.active_table_id, id2);
    }

    #[test]
    fn test_rename_table() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        let id = sheet.tables[0].id;
        sheet.rename_table(id, "My Table".to_string());
        assert_eq!(sheet.tables[0].name, "My Table");
    }

    #[test]
    fn test_rename_table_avoids_conflict() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        let id2 = sheet.add_table("Table 2".to_string(), 3, 3);
        sheet.rename_table(id2, "Table 1".to_string());
        assert_eq!(
            sheet.tables.iter().find(|t| t.id == id2).unwrap().name,
            "Table 1 (2)"
        );
    }

    #[test]
    fn test_table_by_name() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        sheet.add_table("Prices".to_string(), 3, 3);
        assert!(sheet.table_by_name("Prices").is_some());
        assert!(sheet.table_by_name("Nonexistent").is_none());
    }

    #[test]
    fn test_recalculate_all() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        let t = sheet.active_table_mut().unwrap();
        t.set_cell_source(0, 0, "10".to_string());
        t.set_cell_source(0, 1, "=A1*3".to_string());
        sheet.recalculate_all();
        let t = sheet.active_table().unwrap();
        assert_eq!(
            t.cells[&(0, 1)].computed,
            crate::model::cell::CellValue::Number(30.0)
        );
    }

    #[test]
    fn test_cross_table_dependency_tracking() {
        let mut sheet = Sheet::new(1, "S1".to_string());
        let t1_id = sheet.tables[0].id;
        let t2_id = sheet.add_table("Table 2".to_string(), 3, 2);

        // Set up Table 1 with a value
        let t1 = sheet.table_by_id_mut(t1_id).unwrap();
        t1.set_cell_source(0, 0, "42".to_string());

        // Set up Table 2 with a cross-table ref to Table 1
        let t2 = sheet.table_by_id_mut(t2_id).unwrap();
        t2.set_cell_source(0, 0, "=Table 1::A1".to_string());

        // Recalculate all with siblings
        sheet.recalculate_all();

        let t2 = sheet.table_by_id(t2_id).unwrap();
        assert_eq!(
            t2.cells[&(0, 0)].computed,
            crate::model::cell::CellValue::Number(42.0)
        );

        // Now change Table 1 and recalculate dependents
        let t1 = sheet.table_by_id_mut(t1_id).unwrap();
        t1.set_cell_source(0, 0, "100".to_string());
        recalculate_table(sheet.table_by_id_mut(t1_id).unwrap());
        sheet.recalculate_dependents(t1_id);

        let t2 = sheet.table_by_id(t2_id).unwrap();
        assert_eq!(
            t2.cells[&(0, 0)].computed,
            crate::model::cell::CellValue::Number(100.0)
        );
    }
}
