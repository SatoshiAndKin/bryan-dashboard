use serde::{Deserialize, Serialize};

use super::table::{TableId, TableModel};
use super::workbook::unique_name;
use crate::formula::graph::recalculate_table;

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
        let table = TableModel::new(1, "Table 1".to_string(), 6, 5);
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
            .map(|t| t.canvas_y + (t.rows as f32 * 28.0) + 60.0)
            .fold(0.0f32, f32::max);
        table.canvas_y = max_y;
        table.header_row = true;
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
        for table in &mut self.tables {
            recalculate_table(table);
        }
    }
}
